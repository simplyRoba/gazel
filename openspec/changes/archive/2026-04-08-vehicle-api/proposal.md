## Why

The app has a running server, database, and UI shell, but no domain data. Vehicles are the foundational entity — every fill-up, stat, and chart attaches to a vehicle. Before any tracking features can be built, we need a vehicles table and a full CRUD API.

## What Changes

- **Migration**: `vehicles` table with id, name, make, model, year, fuel_type, notes, created_at, updated_at
- **Separate Row and Response types**: `VehicleRow` (sqlx, DB columns) and `Vehicle` (serde, API shape) with a `From` impl. Keeps DB representation decoupled from API contract.
- **Shared SQL const**: `VEHICLE_SELECT` fragment reused across list, get, create (re-fetch), and update (re-fetch) handlers
- **CRUD endpoints**:
  - `GET /api/vehicles` — list all vehicles, ordered by name
  - `GET /api/vehicles/:id` — single vehicle by ID
  - `POST /api/vehicles` — create with validation, returns 201
  - `PUT /api/vehicles/:id` — full update (all fields required)
  - `PATCH /api/vehicles/:id` — partial update with `Option<Option<T>>` semantics for nullable fields (absent = keep, null = clear, value = set)
  - `DELETE /api/vehicles/:id` — delete vehicle. Rejects with 409 Conflict if the vehicle has fill-ups (fill-ups are too important to cascade-delete silently)
- **`deserialize_nullable` helper**: Moved to `src/api/error.rs` (or a shared util) so fill-ups can reuse it later
- **Boundary validation**: Required name (non-empty after trim), fuel_type enum validation, year range validation
- **Integration tests**: Full coverage for all endpoints — CRUD, validation errors, not-found, bad JSON, PATCH null-clearing

## Capabilities

### New Capabilities

- `vehicle-crud`: Vehicle CRUD API — endpoints, request/response types, validation, database operations

### Modified Capabilities

- `api-error-handling`: Add `deserialize_nullable` helper to the error module (shared utility for PATCH semantics across all future domain APIs). Add new error codes for vehicle validation.

## Impact

- **Dependencies**: No new crates (sqlx, serde, chrono, uuid already present)
- **Migrations**: New `migrations/YYYYMMDD_vehicles.sql`
- **Code created**: `src/api/vehicles.rs` (handlers, types, validation, SQL const)
- **Code modified**: `src/api/mod.rs` (register vehicle routes), `src/api/error.rs` (add `deserialize_nullable`, new error codes)
- **Tests created**: `tests/vehicles.rs` (integration tests for all endpoints)
- **Frontend**: No changes (vehicle management UI is a separate change)
