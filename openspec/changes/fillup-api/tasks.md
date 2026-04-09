## 1. Database migration

- [x] 1.1 Create `migrations/YYYYMMDDHHMMSS_fillups.sql` with `fillups` table (id, vehicle_id FK, date, odometer, fuel_amount, fuel_unit, cost, currency, is_full_tank, is_missed, station, notes, created_at, updated_at) and indexes on `(vehicle_id)` and `(vehicle_id, date DESC)`

## 2. Types and shared SQL

- [x] 2.1 Create `src/api/fillups.rs` with `FillupRow` (sqlx::FromRow), `Fillup` (Serialize response), and `From<FillupRow> for Fillup` impl (map INTEGER 0/1 to bool for `is_full_tank` and `is_missed`)
- [x] 2.2 Add `CreateFillup` and `UpdateFillup` request structs with appropriate field types and defaults
- [x] 2.3 Add `FILLUP_SELECT` shared SQL const with all columns from the fillups table

## 3. Validation functions

- [x] 3.1 Add `validate_fillup_date` — require non-empty trimmed string
- [x] 3.2 Add `validate_fuel_amount` — require presence and value > 0
- [x] 3.3 Add `validate_odometer` — when provided, check >= max existing odometer for vehicle (accept optional `exclude_id` for updates)
- [x] 3.4 Add `validate_cost` — when provided, check >= 0
- [x] 3.5 Add `ensure_vehicle_exists` helper — `SELECT 1 FROM vehicles WHERE id = ?`, return `ApiError::NotFound("VEHICLE_NOT_FOUND")` if missing

## 4. CRUD handlers

- [x] 4.1 Implement `list` handler — verify vehicle exists, query fillups for vehicle_id sorted by date DESC, return `Json<Vec<Fillup>>`
- [x] 4.2 Implement `get` handler — verify vehicle exists, fetch fillup by id AND vehicle_id, return `Json<Fillup>` or 404 `FILLUP_NOT_FOUND`
- [x] 4.3 Implement `create` handler — verify vehicle exists, validate fields (date, fuel_amount, odometer, cost), INSERT RETURNING id, re-SELECT with shared SQL, return `(CREATED, Json<Fillup>)`
- [x] 4.4 Implement `update` handler — verify vehicle exists, verify fillup exists, validate fields (including odometer with exclude_id), UPDATE, re-SELECT, return `Json<Fillup>`
- [x] 4.5 Implement `delete` handler — verify vehicle exists, DELETE where id AND vehicle_id, check rows_affected for 404, return `NO_CONTENT`

## 5. Route registration and error codes

- [x] 5.1 Add `pub mod fillups;` to `src/api/mod.rs` and register routes: `/vehicles/{vehicle_id}/fillups` (GET, POST) and `/vehicles/{vehicle_id}/fillups/{id}` (GET, PUT, DELETE)
- [x] 5.2 Add fill-up error codes to `default_message` in `src/api/error.rs`: `FILLUP_NOT_FOUND`, `FILLUP_DATE_REQUIRED`, `FILLUP_FUEL_AMOUNT_REQUIRED`, `FILLUP_INVALID_FUEL_AMOUNT`, `FILLUP_INVALID_ODOMETER`, `FILLUP_INVALID_COST`

## 6. Vehicle delete guard

- [x] 6.1 Modify the `delete` handler in `src/api/vehicles.rs` to check for existing fill-ups before deleting — `SELECT EXISTS(SELECT 1 FROM fillups WHERE vehicle_id = ?)` — return `409 Conflict` with `VEHICLE_HAS_FILLUPS` if any exist

## 7. Integration tests

- [x] 7.1 Create `tests/fillups.rs` with `mod common;` and a `create_vehicle` + `create_fillup` helper function
- [x] 7.2 Add tests for list endpoint: list with fill-ups, list empty, list for non-existent vehicle
- [x] 7.3 Add tests for get endpoint: get existing, get non-existent fillup, get for non-existent vehicle
- [x] 7.4 Add tests for create endpoint: create with all fields, create with required fields only, create for non-existent vehicle
- [x] 7.5 Add tests for create validation: missing date, empty date, missing fuel_amount, zero fuel_amount, negative cost, odometer lower than previous, null odometer accepted
- [x] 7.6 Add tests for update endpoint: successful update, update non-existent fillup, update for non-existent vehicle, update validation (odometer excludes self)
- [x] 7.7 Add tests for delete endpoint: successful delete, delete non-existent fillup, delete for non-existent vehicle
- [x] 7.8 Add test for vehicle delete guard: create vehicle with fill-ups, attempt delete, assert 409 with `VEHICLE_HAS_FILLUPS`; delete fill-ups first, then vehicle delete succeeds

## 8. Verify

- [x] 8.1 Run `cargo fmt -- --check` and fix any formatting issues
- [x] 8.2 Run `cargo clippy -- -D warnings` and fix any lint warnings
- [x] 8.3 Run `cargo test` and ensure all tests pass (existing + new)
