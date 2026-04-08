## 1. Migration

- [x] 1.1 Create `migrations/YYYYMMDD_vehicles.sql` with `CREATE TABLE vehicles` (id INTEGER PRIMARY KEY, name TEXT NOT NULL, make TEXT, model TEXT, year INTEGER, fuel_type TEXT NOT NULL DEFAULT 'gasoline', notes TEXT, created_at TEXT NOT NULL DEFAULT (datetime('now')), updated_at TEXT NOT NULL DEFAULT (datetime('now')))

## 2. Shared utilities

- [x] 2.1 Add `deserialize_nullable` function to `src/api/error.rs` — serde deserializer for `Option<Option<T>>` PATCH semantics
- [x] 2.2 Add vehicle error codes to `default_message()` in `src/api/error.rs`: `VEHICLE_NOT_FOUND`, `VEHICLE_NAME_REQUIRED`, `VEHICLE_INVALID_FUEL_TYPE`, `VEHICLE_INVALID_YEAR`, `VEHICLE_HAS_FILLUPS`

## 3. Vehicle types

- [x] 3.1 Create `src/api/vehicles.rs` with `VehicleRow` (sqlx::FromRow) and `Vehicle` (Serialize) structs
- [x] 3.2 Implement `From<VehicleRow> for Vehicle`
- [x] 3.3 Define `VEHICLE_SELECT` const with the shared SELECT column list
- [x] 3.4 Define `VALID_FUEL_TYPES` const: `gasoline`, `diesel`, `electric`, `hybrid`, `lpg`, `cng`, `other`
- [x] 3.5 Define `CreateVehicle` request struct (name required, rest optional with defaults)
- [x] 3.6 Define `UpdateVehicle` request struct (all fields present for PUT)
- [x] 3.7 Define `PatchVehicle` request struct with `Option<Option<T>>` + `deserialize_nullable` for nullable fields

## 4. Validation

- [x] 4.1 Implement `validate_vehicle_name(name: &str)` — rejects empty/whitespace-only, returns `VEHICLE_NAME_REQUIRED`
- [x] 4.2 Implement `validate_fuel_type(fuel_type: &str)` — checks against `VALID_FUEL_TYPES`, returns `VEHICLE_INVALID_FUEL_TYPE`
- [x] 4.3 Implement `validate_year(year: Option<i64>)` — checks 1900-2100 range when present, returns `VEHICLE_INVALID_YEAR`

## 5. Handlers

- [x] 5.1 `list` handler: `GET /api/vehicles` — query with `VEHICLE_SELECT ORDER BY name`, return `Vec<Vehicle>`
- [x] 5.2 `get` handler: `GET /api/vehicles/{id}` — query with `VEHICLE_SELECT WHERE id = ?`, return `Vehicle` or 404
- [x] 5.3 `create` handler: `POST /api/vehicles` — validate name + fuel_type + year, INSERT, re-fetch with `VEHICLE_SELECT`, return 201
- [x] 5.4 `update` handler: `PUT /api/vehicles/{id}` — check exists (404), validate all fields, UPDATE all columns, re-fetch, return 200
- [x] 5.5 `patch` handler: `PATCH /api/vehicles/{id}` — fetch current, merge with `Option<Option<T>>` unwrap_or pattern, validate merged values, UPDATE, re-fetch, return 200
- [x] 5.6 `delete` handler: `DELETE /api/vehicles/{id}` — check exists (404), DELETE, return 204

## 6. Route registration

- [x] 6.1 Add `pub mod vehicles;` to `src/api/mod.rs`
- [x] 6.2 Register routes: `/vehicles` (GET + POST), `/vehicles/{id}` (GET + PUT + PATCH + DELETE)

## 7. Integration tests

- [x] 7.1 Create `tests/vehicles.rs` with test helpers
- [x] 7.2 Test list empty returns `[]`
- [x] 7.3 Test create with all fields returns 201 with correct shape
- [x] 7.4 Test create with only name returns 201 with defaults (fuel_type=gasoline, make/model/year/notes=null)
- [x] 7.5 Test create without name returns 422 `VEHICLE_NAME_REQUIRED`
- [x] 7.6 Test create with empty name returns 422 `VEHICLE_NAME_REQUIRED`
- [x] 7.7 Test create with invalid fuel_type returns 422 `VEHICLE_INVALID_FUEL_TYPE`
- [x] 7.8 Test create with invalid year returns 422 `VEHICLE_INVALID_YEAR`
- [x] 7.9 Test get existing vehicle returns 200
- [x] 7.10 Test get non-existent returns 404 `VEHICLE_NOT_FOUND`
- [x] 7.11 Test update (PUT) changes fields and bumps updated_at
- [x] 7.12 Test patch only updates sent fields, preserves others
- [x] 7.13 Test patch with `null` clears nullable field
- [x] 7.14 Test delete returns 204, subsequent get returns 404
- [x] 7.15 Test delete non-existent returns 404 `VEHICLE_NOT_FOUND`
- [x] 7.16 Test malformed JSON returns 400 `INVALID_REQUEST_BODY`
- [x] 7.17 Test list after creating multiple vehicles returns them ordered by name

## 8. Verification

- [x] 8.1 Run `cargo fmt -- --check` and fix any formatting issues
- [x] 8.2 Run `cargo clippy -- -D warnings` and fix all warnings
- [x] 8.3 Run `cargo test` and verify all tests pass
- [x] 8.4 Run `npm run check --prefix ui` to verify UI still passes
