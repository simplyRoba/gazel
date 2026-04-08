## Why

gazel is currently an empty scaffold — `main.rs` prints a startup message and exits. No HTTP server, no database, no API surface. Every planned feature (vehicles, fill-ups, stats, UI) depends on having a running Axum server, a SQLite database with migration support, embedded SPA serving, and a consistent error handling contract. This is the prerequisite for all other work.

## What Changes

- **Add core runtime dependencies**: Axum, SQLx (SQLite), rust-embed, serde, tower-http, uuid, chrono
- **Environment-based configuration**: Read `GAZEL_PORT`, `GAZEL_DB_PATH`, `GAZEL_LOG_LEVEL` from env with sensible defaults, using a testable `ConfigSource` trait
- **SQLite database lifecycle**: Connection pool with WAL journal mode, auto-create DB file and parent dirs, run migrations on startup
- **Health endpoint**: `GET /health` returns `200 OK` with version info after verifying DB connectivity
- **Embedded SPA serving**: rust-embed serves the SvelteKit build at all non-API routes, falling back to `index.html` for client-side routing
- **Router infrastructure**: Axum router with `/api` namespace, access-log middleware (method, path, status, latency), and graceful shutdown (SIGINT/SIGTERM)
- **Application state**: `AppState` struct with `FromRef` impl so handlers can extract `SqlitePool` directly
- **API error contract**: `ApiError` enum mapping to JSON `{ "code": string, "message": string }` responses, `db_error()` helper, and `JsonBody<T>` custom extractor for parse-error handling
- **Test harness**: Integration test helpers (`test_app()`, `json_request()`, `body_json()`) using in-memory SQLite, plus `tests/ui.rs` bridge running `npm test`

## Capabilities

### New Capabilities

- `http-server`: Axum server lifecycle — binding, graceful shutdown, access logging, and embedded SPA serving
- `database`: SQLite connection pool, WAL configuration, and migration execution on startup
- `configuration`: Environment-variable-based configuration with typed defaults and testable abstraction
- `api-error-handling`: Consistent API error response contract, custom JSON body extractor, and database error mapping
- `health-check`: Health endpoint verifying database connectivity and reporting version

### Modified Capabilities

_(none — no existing specs)_

## Impact

- **Dependencies**: `Cargo.toml` gains axum, sqlx, rust-embed, serde/serde_json, tower/tower-http, uuid, chrono, tracing-subscriber env-filter. Dev-dep: tempfile.
- **Code**: `src/main.rs` rewritten. New modules: `src/config.rs`, `src/db.rs`, `src/server.rs`, `src/state.rs`, `src/embedded.rs`, `src/api/mod.rs`, `src/api/error.rs`
- **Tests**: New `tests/common/mod.rs` (shared harness), `tests/health.rs` (health endpoint), `tests/ui.rs` (UI test bridge)
- **Migrations**: New `migrations/` directory with initial empty bootstrap migration
- **Build**: `build.rs` already handles UI build — no changes needed
- **Docker**: Existing `Dockerfile` and `docker-compose.yml` already expect the binary shape — no changes needed
