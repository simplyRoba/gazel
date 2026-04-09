## Why

Vehicles exist but there is no way to record fuel fill-ups against them. Fill-ups are the primary data-entry surface of gazel -- every downstream feature (efficiency calculations, cost stats, charts, dashboard) depends on having fill-up records. This is the next chunk in the 1.0 roadmap and unblocks chunks 7-11.

## What Changes

- Add a `fillups` table with a foreign-key relationship to `vehicles`, storing date, odometer reading, fuel amount/unit, cost/currency, full-tank and missed-fill flags, station name, and notes.
- Implement nested REST endpoints under `/api/vehicles/:id/fillups` for full CRUD (list, get, create, update, delete).
- List endpoint returns fill-ups sorted by date descending (most recent first).
- Create endpoint validates required fields and enforces odometer ordering (new reading must be greater than the previous for the same vehicle).
- Update endpoint supports full replacement (PUT) with the same validation rules.
- Delete endpoint removes a single fill-up by ID.
- Reject deletion of a vehicle that has fill-ups (enforce the existing `VEHICLE_HAS_FILLUPS` error code already defined in the error-handling spec).
- Add integration tests covering all endpoints and validation edge cases.

## Capabilities

### New Capabilities

- `fillup-crud`: CRUD endpoints for fuel fill-ups nested under a vehicle, including the database migration, request/response types, validation rules, and error codes.

### Modified Capabilities

- `vehicle-crud`: Add enforcement of the delete-rejection rule when a vehicle has fill-ups (the scenario already exists in the spec but the backend does not yet enforce it since fill-ups did not exist).
- `api-error-handling`: Add fill-up-specific error codes and their default messages.

## Impact

- **Database**: New `fillups` migration in `migrations/`. Foreign key from `fillups.vehicle_id` to `vehicles.id`.
- **Backend**: New handler module `src/api/fillups.rs` with routes registered in `src/api/mod.rs`. New row/response types, validation functions, and SQL queries following the vehicle-crud patterns.
- **Error handling**: New error codes (`FILLUP_NOT_FOUND`, `FILLUP_DATE_REQUIRED`, `FILLUP_ODOMETER_REQUIRED`, `FILLUP_FUEL_AMOUNT_REQUIRED`, `FILLUP_INVALID_ODOMETER`, etc.) added to `default_message` in `src/api/error.rs`.
- **Vehicle delete**: `DELETE /api/vehicles/:id` must now check for existing fill-ups and return 409 with `VEHICLE_HAS_FILLUPS` instead of deleting.
- **Tests**: New integration test file `tests/fillups.rs`. Existing vehicle delete tests may need updating to cover the has-fillups rejection path.
- **No frontend changes** -- UI is a separate change (chunk 7).
- **No new dependencies** -- uses existing sqlx, axum, serde stack.
