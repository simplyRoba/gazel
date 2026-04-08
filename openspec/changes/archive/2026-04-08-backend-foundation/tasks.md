## 1. Dependencies and project setup

- [x] 1.1 Add runtime dependencies to `Cargo.toml`: axum, sqlx (runtime-tokio, sqlite), rust-embed, serde (derive), serde_json, tower, tower-http (compression-gzip), chrono, uuid (v4)
- [x] 1.2 Add dev-dependency: tempfile
- [x] 1.3 Add initial empty bootstrap migration file in `migrations/`

## 2. Configuration

- [x] 2.1 Create `src/config.rs` with `ConfigSource` trait, `EnvConfigSource`, and `parse_or<T>` helper
- [x] 2.2 Implement `Config::load()` and `Config::load_from()` reading `GAZEL_PORT`, `GAZEL_DB_PATH`, `GAZEL_LOG_LEVEL` with defaults (4110, `/data/gazel.db`, `info`)
- [x] 2.3 Write unit tests for config: defaults, custom values, invalid values fall back to defaults, mock config source

## 3. Database

- [x] 3.1 Create `src/db.rs` with `create_pool()`: WAL journal mode, 5s busy timeout, max 5 connections, create-if-missing, auto-create parent dirs (guard for `:memory:`)
- [x] 3.2 Add `run_migrations()` function calling `sqlx::migrate!().run(pool)`

## 4. Application state

- [x] 4.1 Create `src/state.rs` with `AppState` struct holding `SqlitePool`
- [x] 4.2 Implement `FromRef<AppState> for SqlitePool`

## 5. API error handling

- [x] 5.1 Create `src/api/error.rs` with `ApiError` enum: `NotFound`, `Validation`, `Conflict`, `BadRequest`, `InternalError` — each carrying `&'static str` error code
- [x] 5.2 Implement `IntoResponse` for `ApiError` mapping variants to HTTP status codes and JSON `{ "code", "message" }` body
- [x] 5.3 Add `default_message()` function mapping error codes to human-readable strings
- [x] 5.4 Add `db_error(sqlx::Error) -> ApiError` helper that logs at error level and returns `InternalError("INTERNAL_ERROR")`
- [x] 5.5 Create `JsonBody<T>` extractor that wraps `axum::Json<T>` and returns `ApiError::BadRequest("INVALID_REQUEST_BODY")` on parse failure
- [x] 5.6 Create `src/api/mod.rs` with empty API sub-router (placeholder for future domain routes)

## 6. Embedded SPA handler

- [x] 6.1 Create `src/embedded.rs` with rust-embed `Assets` struct pointing to `ui/build/`
- [x] 6.2 Implement `static_handler` with two-step lookup: exact path match, then `index.html` fallback
- [x] 6.3 Infer `Content-Type` from file extension via `mime_guess`

## 7. Server and router

- [x] 7.1 Create `src/server.rs` with `router()` function: mount `/health`, nest `/api` sub-router, set embedded SPA as fallback
- [x] 7.2 Implement `access_log` middleware: debug-level log with method, path, status, duration in ms
- [x] 7.3 Implement `serve()` with `TcpListener` binding and `axum::serve` with graceful shutdown
- [x] 7.4 Implement `shutdown_signal()` awaiting SIGINT or SIGTERM via `tokio::signal` and `tokio::select!`
- [x] 7.5 Implement health handler: `GET /health` queries `SELECT 1`, returns `{ "status": "ok", "version": "..." }` on success, `503` with `{ "status": "unhealthy" }` on failure

## 8. Main entrypoint

- [x] 8.1 Rewrite `src/main.rs`: config → tracing init → create pool → run migrations → build state → build router → serve
- [x] 8.2 Add module declarations for config, db, server, state, embedded, api

## 9. Test infrastructure

- [x] 9.1 Create `tests/common/mod.rs` with `test_pool()` (in-memory, max 1 connection, runs migrations), `test_app()` (returns Router), `json_request()`, `body_json()`
- [x] 9.2 Create `tests/health.rs`: test healthy response (200, status ok, version), test correct JSON shape
- [x] 9.3 Create `tests/ui.rs`: auto-install node_modules if missing, shell out to `npm run test`, skippable with `--skip ui_tests`

## 10. Verification

- [x] 10.1 Run `cargo fmt -- --check` and fix any formatting issues
- [x] 10.2 Run `cargo clippy -- -D warnings` and fix all warnings
- [x] 10.3 Run `npm run format:check --prefix ui` and `npm run lint --prefix ui` and `npm run check --prefix ui`
- [x] 10.4 Run `cargo test` and verify all tests pass (backend + UI bridge)
