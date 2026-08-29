## 1. Backend

- [x] 1.1 Add integration tests for default/custom limits, date-and-ID ordering, continuation, terminal/empty pages, invalid inputs/error codes, and missing vehicles.
- [x] 1.2 Add typed page-query, cursor, and response structures; ensure query parsing failures use JSON `ApiError` responses.
- [x] 1.3 Replace unbounded listing with `limit + 1` keyset retrieval and opaque cursor generation.

## 2. Frontend

- [x] 2.1 Update API tests and types for the single `FillupPage` response contract.
- [x] 2.2 Add store tests for per-vehicle pages, append/de-duplication, generation races, guards, failure/retry, exhaustion, switching, and mutation refresh.
- [x] 2.3 Refactor the store to implement tested page-chain and generation behavior.
- [x] 2.4 Add dashboard tests for the desktop scroll root, mobile viewport, observer cleanup, loading, retry, and exhaustion.
- [x] 2.5 Implement the endless-scroll sentinel and continuation states without changing loaded-card behavior.
- [x] 2.6 Update form-helper tests so loaded recent entries are used without eager continuation.

## 3. Verification

- [x] 3.1 Update direct API documentation for the breaking list envelope if applicable.
- [x] 3.2 Run focused backend and frontend pagination tests.
- [x] 3.3 Run `cargo fmt -- --check`, `cargo clippy -- -D warnings`, `npm run format:check --prefix ui`, `npm run lint --prefix ui`, `npm run check --prefix ui`, and `cargo test`.
