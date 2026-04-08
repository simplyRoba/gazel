## Context

gazel's `main.rs` currently initializes tracing and prints a version string. There is no HTTP server, database, API surface, or test infrastructure. The existing `Cargo.toml` has tokio, tracing, and tracing-subscriber. The `build.rs` already handles building and embedding the SvelteKit SPA with `SKIP_UI_BUILD` support.

The sibling project [flowl](https://github.com/simplyRoba/flowl) uses the identical stack (Axum + SQLx/SQLite + rust-embed) and has mature, proven patterns for every module this change introduces. The design follows those patterns closely to avoid reinventing solutions and to maintain consistency across the author's projects.

## Goals / Non-Goals

**Goals:**

- A running HTTP server that can serve API routes and the embedded SPA
- A SQLite database that auto-creates, configures WAL mode, and runs migrations on startup
- Environment-based configuration with typed defaults, testable without real env vars
- A consistent API error contract that all future endpoints will use
- A health endpoint that verifies actual database connectivity
- An integration test harness that enables fast, isolated backend tests
- Graceful shutdown on SIGINT/SIGTERM

**Non-Goals:**

- No domain endpoints yet (vehicles, fill-ups, stats) — those come in later changes
- No authentication or authorization — gazel runs on trusted networks
- No HTTPS termination — handled by a reverse proxy
- No database connection retry/backoff — single-user app, fail fast on startup is acceptable
- No request rate limiting — no auth means no untrusted callers
- No CORS configuration — SPA is served from the same origin

## Decisions

### Module structure

Adopt flowl's flat module layout under `src/`:

```
src/
  main.rs          -- bootstrap: config → pool → migrations → state → router → serve
  config.rs        -- env-based config with ConfigSource trait
  db.rs            -- pool creation and migration runner
  server.rs        -- router assembly, middleware, serve + shutdown
  state.rs         -- AppState struct
  embedded.rs      -- rust-embed SPA handler
  api/
    mod.rs         -- API sub-router (empty for now, will grow per-domain)
    error.rs       -- ApiError enum, JsonBody extractor
```

**Rationale:** Flat modules keep navigation simple in a single-crate project. The `api/` subdirectory isolates all HTTP-boundary code. This mirrors flowl's structure exactly, which is proven to scale to 70+ releases.

**Alternative considered:** Feature-grouped modules (e.g., `src/health/mod.rs`, `src/health/handler.rs`). Rejected — too much ceremony for a small codebase; adds file navigation overhead without benefit.

### Configuration via `ConfigSource` trait

```rust
pub trait ConfigSource {
    fn get(&self, key: &str) -> Option<String>;
}
```

Production uses `EnvConfigSource` (reads `std::env::var`). Tests use a `HashMap`-based mock. A `parse_or<T: FromStr>(source, key, default)` helper handles typed parsing with fallback.

**Rationale:** Makes config fully testable without `std::env::set_var` (which is unsound in multi-threaded tests). Proven in flowl's `src/config.rs`.

**Alternative considered:** Direct `std::env::var` calls with `#[cfg(test)]` overrides. Rejected — fragile in parallel test execution and less explicit.

### SQLite pool configuration

- **Journal mode:** WAL (Write-Ahead Logging) — allows concurrent reads during writes
- **Busy timeout:** 5 seconds — prevents immediate failure under brief contention
- **Max connections:** 5 — sufficient for single-user; WAL benefits from limited parallelism
- **Create-if-missing:** true — first run creates the DB file automatically
- **Parent directory creation:** auto-create parent dirs for the DB path (guard for `:memory:`)

**Rationale:** These are SQLite best practices for an embedded, single-user workload. Identical to flowl's `src/db.rs`.

### API error contract

```rust
pub enum ApiError {
    NotFound(&'static str),
    Validation(&'static str),
    Conflict(&'static str),
    BadRequest(&'static str),
    InternalError(&'static str),
}
```

Each variant carries a `&'static str` error code (e.g., `"VEHICLE_NOT_FOUND"`). The `IntoResponse` impl maps to HTTP status + JSON `{ "code": "...", "message": "..." }`. A `default_message()` function maps codes to human-readable English strings (later replaceable with i18n keys).

A `db_error(sqlx::Error) -> ApiError` helper logs the underlying error at `error` level and returns `InternalError("INTERNAL_ERROR")` — never leaking DB details to the client.

**Rationale:** Static error codes enable future i18n (`resolveError()` on the frontend can map codes to translations). The `&'static str` avoids allocations. Matches flowl's pattern in `src/api/error.rs`.

**Alternative considered:** Dynamic `String` messages per error. Rejected — less structured, harder to match on the frontend, and allocates unnecessarily.

### `JsonBody<T>` custom extractor

Wraps `axum::Json<T>` but rejects malformed request bodies with `ApiError::BadRequest("INVALID_REQUEST_BODY")` instead of axum's default plain-text error. This keeps all error responses in the same JSON shape.

**Rationale:** Without this, a client sending malformed JSON gets a different error format than all other errors, making frontend error handling inconsistent.

### Health endpoint

`GET /health` lives outside the `/api` namespace (consistent with Docker `HEALTHCHECK` and load balancer conventions). It queries `SELECT 1` against the pool and returns:

```json
{ "status": "ok", "version": "0.1.0" }
```

On DB failure: `503 Service Unavailable` with `{ "status": "unhealthy" }`.

**Rationale:** A real DB query catches connection pool exhaustion, file corruption, and disk issues — not just "the process is alive." The existing `Dockerfile` `HEALTHCHECK` already curls `/health`.

### Embedded SPA handler

Registered as `Router::fallback()` so API routes always take priority. Uses a two-step lookup:

1. Try exact path match against rust-embed assets
2. Fall back to `index.html` for SPA client-side routing

Content-Type is inferred via `mime_guess` from the file extension.

**Rationale:** The fallback approach means any new API route is automatically prioritized. No need to maintain a list of SPA routes. Identical to flowl's `src/embedded.rs`.

### Access log middleware

A `tower` `from_fn` middleware that logs every request at `debug` level:

```
GET /api/vehicles → 200 (12ms)
```

Captures method, path, status code, and elapsed time.

**Rationale:** Debug-level keeps production logs clean while giving full visibility when `GAZEL_LOG_LEVEL=debug`. Middleware approach means every route is covered automatically.

### Graceful shutdown

Listens for both `SIGINT` (Ctrl+C) and `SIGTERM` (Docker stop). Uses `tokio::signal` with `tokio::select!` to await either signal, then lets axum's `graceful_shutdown` drain in-flight requests.

**Rationale:** Docker sends `SIGTERM` on `docker stop`; developers use Ctrl+C. Both must trigger clean shutdown to avoid SQLite WAL corruption on abrupt exit.

### Test infrastructure

**`tests/common/mod.rs`:**
- `test_pool()` — in-memory SQLite pool (`":memory:"`) with max 1 connection, runs all migrations
- `test_app()` — creates pool + state + router, returns `Router`
- `json_request(method, uri, body)` — builds a `Request<Body>` with JSON content-type
- `body_json(response)` — reads response body into `serde_json::Value`

Tests use `tower::ServiceExt::oneshot` to call the router without a TCP listener. Each test gets its own in-memory database — fully isolated, no cleanup needed.

**`tests/ui.rs`:**
- Runs `npm install` if `ui/node_modules` is missing
- Shells out to `npm run test` in the `ui/` directory
- Skippable with `cargo test -- --skip ui_tests`

**Rationale:** In-memory per-test databases are fast and side-effect-free. The `oneshot` approach avoids port conflicts. The UI bridge test ensures `cargo test` covers the full stack. All patterns proven in flowl's test suite.

### Dependencies

| Crate | Purpose |
|---|---|
| `axum` 0.8 | HTTP framework |
| `sqlx` 0.8 (runtime-tokio, sqlite) | Async SQLite with compile-time query checking |
| `rust-embed` 8 | Embed SvelteKit build into binary |
| `serde` 1 (derive) | Serialization framework |
| `serde_json` 1 | JSON serialization |
| `tower` 0.5 | Middleware and service utilities |
| `tower-http` 0.6 (compression-gzip) | HTTP-specific middleware (gzip response compression) |
| `chrono` 0.4 | Date/time types (needed soon for fill-ups, add now to avoid churn) |
| `uuid` 1 (v4) | UUID generation (needed soon for entity IDs, add now) |
| `tempfile` 3 (dev-dependency) | Temporary directories in integration tests |

**Not adding yet:** `axum-extra` (no query extraction needed), `mime_guess` (re-export from `rust-embed` may suffice — verify during implementation).

## Risks / Trade-offs

**Single-connection in-memory test pools may hide concurrency bugs** → Acceptable for a single-user app. If concurrency issues arise later, switch test pools to file-backed with 5 connections.

**No retry on DB connection failure at startup** → The app fails fast if the database is unreachable. For a self-hosted single-user app this is the right behavior — the user can restart. A managed service would want retry/backoff.

**`&'static str` error codes are not extensible at runtime** → All error codes are known at compile time. This is a feature, not a limitation — it prevents typos and enables exhaustive matching. New codes require a code change, which is appropriate.

**Adding `chrono` and `uuid` before they're strictly needed** → Minor binary size cost. Avoids a dependency churn commit later when vehicles/fill-ups land. Both are near-certain to be needed.
