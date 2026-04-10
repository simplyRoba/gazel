## 1. Export types and serialization

- [x] 1.1 Define `ExportData`, `ExportVehicle`, `ExportFillup` structs with serde `Serialize`/`Deserialize` in a new `src/api/export.rs` module
- [x] 1.2 Implement `version` field using `env!("CARGO_PKG_VERSION")` and `exported_at` as RFC 3339 UTC timestamp
- [x] 1.3 Write a helper that queries all vehicles with their fill-ups and assembles an `ExportData` (fill-ups sorted by date ascending per vehicle)

## 2. Export endpoints

- [x] 2.1 Add `GET /api/export` handler returning full export JSON with `Content-Disposition: attachment; filename="gazel-export.json"`
- [x] 2.2 Add `GET /api/vehicles/:id/export` handler returning single-vehicle export with `Content-Disposition` using kebab-cased vehicle name
- [x] 2.3 Register both routes in `src/api/mod.rs`
- [x] 2.4 Write integration tests in `tests/export.rs`: empty DB export, populated export, single vehicle export, vehicle not found, fill-up ordering, content-disposition headers

## 3. Import types and validation

- [x] 3.1 Define import request types and `ImportResult`/`ImportPreview` response structs in a new `src/api/import.rs` module
- [x] 3.2 Implement `check_version()` that compares major.minor of export version against `CARGO_PKG_VERSION`
- [x] 3.3 Implement import validation: required vehicle fields (name), fill-up field constraints (non-negative odometer/cost/fuel_amount), and schema structure checks

## 4. Import endpoint -- replace mode

- [x] 4.1 Add `POST /api/import` handler with `mode` and `preview` query parameters (default mode = `replace`)
- [x] 4.2 Implement replace mode: delete all vehicles + fill-ups in a transaction, insert imported data, return summary
- [x] 4.3 Implement preview mode (`preview=true`): validate and return counts without writing
- [x] 4.4 Register route in `src/api/mod.rs` with 10 MB body size limit
- [x] 4.5 Write integration tests in `tests/import.rs`: valid replace import, preview dry run, version mismatch (major, minor), patch version allowed, missing version field, malformed JSON, validation errors (missing name, negative odometer), atomicity (failed import preserves existing data), body size limit

## 5. Import endpoint -- merge mode

- [x] 5.1 Implement merge mode: match vehicles by name (case-insensitive), update matched vehicle fields, insert unmatched vehicles
- [x] 5.2 Implement fill-up merge: match by (vehicle, date, odometer), skip matched fill-ups, insert unmatched
- [x] 5.3 Implement merge preview with `vehicles_new`, `vehicles_existing`, `fillups_new`, `fillups_existing` counts
- [x] 5.4 Write integration tests for merge: new vehicles inserted, existing vehicles updated, duplicate fill-ups skipped, new fill-ups inserted, merge preview counts

## 6. Frontend API client

- [x] 6.1 Add export API functions in `ui/src/lib/api.ts`: `exportAll()`, `exportVehicle(id)` returning blob/download
- [x] 6.2 Add import API functions: `previewImport(file, mode)`, `importData(file, mode)` with typed response interfaces

## 7. Settings page -- export and import UI

- [x] 7.1 Add "Data" section to the settings page with "Export data" download button
- [x] 7.2 Add "Import data" file picker with mode selector (Replace / Merge) and preview/confirm flow
- [x] 7.3 Show preview summary after file selection (vehicle/fill-up counts, replace warning)
- [x] 7.4 On confirm: call import API, show success notification, refresh stores
- [x] 7.5 Handle and display import errors (version mismatch, validation errors)
- [x] 7.6 Add per-vehicle export action (context menu or button on vehicle cards)

## 8. Frontend tests

- [x] 8.1 Write vitest tests for export/import API client functions (mock fetch)
- [x] 8.2 Write vitest tests for import preview and confirm flow components

## 9. Verification and cleanup

- [x] 9.1 Run `cargo fmt -- --check` and `cargo clippy -- -D warnings`
- [x] 9.2 Run `npm run format:check --prefix ui`, `npm run lint --prefix ui`, `npm run check --prefix ui`
- [x] 9.3 Run full test suite: `cargo test`
- [x] 9.4 Update README.md data management section if needed
