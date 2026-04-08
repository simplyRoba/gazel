## Context

gazel has a running Axum server with SQLite, an `ApiError` enum with `JsonBody<T>` extractor, integration test harness (`test_app()`, `json_request()`, `body_json()`), and an empty `/api` sub-router. The sibling project flowl has a mature CRUD pattern (`plants.rs`) that we follow closely — separate Row/Response types, shared SQL const, `Option<Option<T>>` PATCH semantics, boundary validation, and `deserialize_nullable`.

This is the first domain API. The patterns established here (file structure, type naming, validation approach, test style) will be replicated for fill-ups and any future entities.

## Goals / Non-Goals

**Goals:**

- A complete, well-tested vehicle CRUD API that serves as the pattern for all future domain APIs
- Clean separation between database representation and API response shape
- Robust PATCH semantics that distinguish absent, null, and present values
- Boundary validation with machine-readable error codes
- Delete protection for vehicles that have fill-ups

**Non-Goals:**

- No pagination (with 1-3 vehicles typical, pagination is unnecessary)
- No filtering or search (list returns all vehicles)
- No soft delete (hard delete only, with fill-up check)
- No frontend changes (vehicle management UI is a separate change)
- No vehicle photos or images

## Decisions

### Database schema

```sql
CREATE TABLE vehicles (
    id         INTEGER PRIMARY KEY,
    name       TEXT    NOT NULL,
    make       TEXT,
    model      TEXT,
    year       INTEGER,
    fuel_type  TEXT    NOT NULL DEFAULT 'gasoline',
    notes      TEXT,
    created_at TEXT    NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT    NOT NULL DEFAULT (datetime('now'))
);
```

**Rationale:** Minimal schema covering the fields from TODO.md. `name` is required (user's label for the vehicle, e.g., "Daily Driver"). `make`, `model`, `year`, and `notes` are optional. `fuel_type` defaults to `gasoline` — the most common case. Timestamps use SQLite `TEXT` with ISO 8601 format (same as flowl).

**Alternative considered:** UUID primary key. Rejected — `INTEGER PRIMARY KEY` is SQLite's rowid alias, faster for joins, and sufficient for a single-user app. UUIDs add complexity without benefit here.

### Fuel type as a validated string enum

Valid values: `gasoline`, `diesel`, `electric`, `hybrid`, `lpg`, `cng`, `other`.

Stored as `TEXT` in SQLite, validated at the API boundary. Not a Rust enum — keeps the database flexible for future values without requiring migrations.

**Rationale:** Same approach as flowl's `light_needs` and `difficulty` fields. Validation at the boundary catches bad input; the database stores the validated string.

### Separate `VehicleRow` and `Vehicle` types

```rust
#[derive(sqlx::FromRow)]
struct VehicleRow { /* DB column names */ }

#[derive(Serialize)]
struct Vehicle { /* API field names */ }

impl From<VehicleRow> for Vehicle { /* map + compute derived fields */ }
```

Currently the fields are identical, but the separation allows future computed fields (e.g., total fill-ups count, last fill-up date) without changing the database query pattern. The `From` impl is the single conversion point.

**Rationale:** Proven pattern from flowl. Adding computed fields later is additive — just add fields to `Vehicle` and compute them in `From`.

### `VEHICLE_SELECT` shared SQL const

```rust
const VEHICLE_SELECT: &str = "SELECT id, name, make, model, year, fuel_type, \
    notes, created_at, updated_at FROM vehicles";
```

Used by list (+ `ORDER BY`), get (+ `WHERE id = ?`), and re-fetch after create/update. Keeps column lists consistent.

**Rationale:** Same pattern as flowl's `PLANT_SELECT`. Single source of truth for the SELECT column list.

### PATCH with `Option<Option<T>>` and `deserialize_nullable`

For nullable fields (`make`, `model`, `year`, `notes`), PATCH requests use three-state semantics:

| JSON field | Rust value | Meaning |
|---|---|---|
| absent | `None` | Keep current value |
| `"field": null` | `Some(None)` | Clear to NULL |
| `"field": "value"` | `Some(Some("value"))` | Set new value |

The `deserialize_nullable` helper (from flowl) is added to `src/api/error.rs` as a shared utility since fill-ups will need the same pattern.

**Rationale:** Without `Option<Option<T>>`, there's no way to distinguish "user didn't send this field" from "user wants to clear this field." This is essential for PATCH semantics. Moving the helper to the error module (rather than per-domain) avoids duplication.

### PUT vs PATCH

Both are supported:
- **PUT** (`UpdateVehicle`): All fields present. Missing optional fields default to their zero value. This is a full replace.
- **PATCH** (`PatchVehicle`): Only sent fields are updated. Uses `Option<Option<T>>` for nullable fields.

The handlers share validation logic but use different request types.

**Rationale:** PUT is simpler for clients that have the full object. PATCH is more efficient for targeted updates (e.g., just renaming a vehicle). flowl only has PATCH (called `update`); we add PUT for completeness since it's trivial once PATCH exists.

### Delete with fill-up protection

```rust
// Check for fill-ups before deleting
let has_fillups = sqlx::query_scalar::<_, bool>(
    "SELECT EXISTS(SELECT 1 FROM fillups WHERE vehicle_id = ?)"
).bind(id).fetch_one(&pool).await?;

if has_fillups {
    return Err(ApiError::Conflict("VEHICLE_HAS_FILLUPS"));
}
```

Returns `409 Conflict` with code `VEHICLE_HAS_FILLUPS` if the vehicle has any fill-ups. The user must delete fill-ups first (or we add a force-delete option later).

**Rationale:** Fill-up data is the primary value of the app — years of fuel tracking shouldn't be silently deleted by removing a vehicle. flowl uses `ON DELETE CASCADE` for care events, but those are low-value; fill-ups are high-value.

**Alternative considered:** `ON DELETE CASCADE` in the migration. Rejected — too dangerous for important data. Also considered soft delete — rejected as premature complexity.

**Note:** The `fillups` table doesn't exist yet. The check query will return false (no fillups) until the fillups migration is added in a later change. The EXISTS query won't error on a missing table — it will error on an unknown column. To handle this gracefully, the check is wrapped in a conditional that only runs if the fillups table exists, OR we simply omit the check until the fillups table is created and add it as a modification in the fillups change. Given that vehicles can be freely deleted until fillups exist, the simpler approach is to **skip the fill-up check for now** and add it when the fillups API is built.

### Validation functions

Following flowl's pattern of small, focused validation functions:

```rust
fn validate_vehicle_name(name: &str) -> Result<(), ApiError>
fn validate_fuel_type(fuel_type: &str) -> Result<(), ApiError>
fn validate_year(year: Option<i64>) -> Result<(), ApiError>
```

Each returns `ApiError::Validation("VEHICLE_*")` with a specific error code. Called at the handler boundary before any database operations.

### Error codes

| Code | Status | When |
|---|---|---|
| `VEHICLE_NOT_FOUND` | 404 | GET/PUT/PATCH/DELETE with unknown ID |
| `VEHICLE_NAME_REQUIRED` | 422 | Name missing or empty after trim |
| `VEHICLE_INVALID_FUEL_TYPE` | 422 | fuel_type not in allowed set |
| `VEHICLE_INVALID_YEAR` | 422 | Year outside 1900-2100 range |
| `VEHICLE_HAS_FILLUPS` | 409 | Delete when vehicle has fill-ups |

Added to `default_message()` in `error.rs`.

### Route registration

```rust
// In src/api/mod.rs
Router::new()
    .route("/vehicles", get(vehicles::list).post(vehicles::create))
    .route("/vehicles/{id}",
        get(vehicles::get)
            .put(vehicles::update)
            .patch(vehicles::patch)
            .delete(vehicles::delete))
```

**Note:** Axum 0.8 uses `{id}` syntax for path parameters (not `:id`).

## Risks / Trade-offs

**Fill-up check deferred** → The delete handler won't check for fill-ups until the fillups table exists. This means vehicles can be freely deleted in the interim. Acceptable because there are no fill-ups to protect yet.

**No pagination** → If someone adds 50+ vehicles, the list endpoint returns them all. Extremely unlikely for a personal fuel tracker (typical: 1-3 vehicles). Pagination can be added later without breaking the API (add optional `?limit=&offset=` query params).

**PUT requires all fields** → Clients must send the complete object. This is standard REST semantics but means the client must first GET the vehicle to do a PUT. PATCH avoids this for partial updates.

**String enum not enforced at DB level** → `fuel_type` is `TEXT`, not a CHECK constraint. Invalid values are caught at the API boundary but could theoretically be inserted via direct SQL. Acceptable for a single-entry-point app.
