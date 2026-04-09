## Context

gazel has a working Vehicle CRUD API (`src/api/vehicles.rs`) with established patterns for type layering (Row → Response via `From`), request structs, validation functions, shared SQL constants, and integration tests. The fill-up API is the second resource and the first with a parent-child relationship (vehicle → fill-ups). It must integrate cleanly with the existing router, error handling, and test infrastructure without introducing new dependencies.

The `VEHICLE_HAS_FILLUPS` error code is already defined in `src/api/error.rs` but not yet enforced because fill-ups didn't exist. The settings system already stores unit preferences that downstream consumers (efficiency calculations, chunk 9) will use, but this change is purely backend CRUD -- no unit conversion logic here.

## Goals / Non-Goals

**Goals:**
- Implement complete CRUD for fill-ups nested under vehicles
- Follow the same handler, type, and test patterns established by the vehicle API
- Enforce referential integrity (vehicle must exist, vehicle delete blocked when fill-ups exist)
- Validate business rules at the API boundary (required fields, odometer ordering)
- Provide a solid foundation for the fill-up logging UI (chunk 7) and stats engine (chunk 9)

**Non-Goals:**
- Pagination -- list endpoint returns all fill-ups for a vehicle; pagination deferred until data volume warrants it
- Fuel efficiency calculations -- that's chunk 9 (depends on fill-ups + settings)
- PATCH (partial update) for fill-ups -- keep it simple with PUT-only updates for now; PATCH can be added later if the UI needs it
- Unit conversion -- fill-ups store raw values with their unit labels; conversion is a display concern
- Frontend changes -- separate change (chunk 7)

## Decisions

### 1. Nested routes under `/api/vehicles/{vehicle_id}/fillups`

Fill-ups are always scoped to a vehicle. Nesting the routes makes this relationship explicit in the URL and simplifies authorization/scoping later.

Routes:
- `GET /api/vehicles/{vehicle_id}/fillups` — list
- `GET /api/vehicles/{vehicle_id}/fillups/{id}` — get single
- `POST /api/vehicles/{vehicle_id}/fillups` — create
- `PUT /api/vehicles/{vehicle_id}/fillups/{id}` — update
- `DELETE /api/vehicles/{vehicle_id}/fillups/{id}` — delete

The path parameter is `vehicle_id` (not `id`) to avoid ambiguity with the fillup `id`. Handlers extract both via `Path((vehicle_id, id))` tuple.

**Alternative considered:** Flat routes (`/api/fillups/:id`). Rejected because it hides the vehicle relationship, requires an extra query param for filtering, and doesn't match the data model's ownership semantics.

### 2. Store raw values with unit labels, no conversion at API level

The `fillups` table stores `fuel_amount` (REAL), `fuel_unit` (TEXT, e.g. "liters", "gallons"), `cost` (REAL), and `currency` (TEXT, e.g. "USD", "EUR"). The API accepts and returns these raw values without conversion.

Unit conversion and formatting are display-layer concerns handled by the frontend's `formatVolume()` / `formatCurrency()` utilities and the future stats engine (chunk 9).

**Alternative considered:** Normalize to a canonical unit (e.g., always store liters). Rejected because it introduces lossy conversion, complicates data import/export, and the settings system already defines user preferences for display.

### 3. Odometer validation: new reading >= previous for same vehicle

On create, the API checks that `odometer` (when provided) is >= the highest existing odometer reading for the same vehicle. This catches obvious data-entry errors (e.g., typing 1,000 instead of 10,000).

On update (PUT), the same check applies but excludes the fill-up being updated from the comparison.

Odometer is optional -- not all users track it. When absent (null), no ordering check is performed.

**Alternative considered:** Strict greater-than only (no equal). Rejected because two fill-ups on the same trip (e.g., split fueling) legitimately share an odometer reading.

### 4. No PATCH endpoint initially

The vehicle API has PATCH with `Option<Option<T>>` semantics. For fill-ups, PUT (full replace) is sufficient for the initial UI, which will load the current fill-up into a form and submit the complete object. PATCH adds complexity for a marginal UX gain at this stage.

### 5. Vehicle existence check in every handler

Every fill-up handler first verifies the `vehicle_id` refers to an existing vehicle, returning 404 with `VEHICLE_NOT_FOUND` if not. This is a single `SELECT 1 FROM vehicles WHERE id = ?` query, cheap on SQLite with integer primary keys.

**Alternative considered:** Rely on the FK constraint to catch invalid vehicle IDs on INSERT. Rejected because GET/list/delete wouldn't naturally trigger an FK check, leading to inconsistent 404 behavior.

### 6. Vehicle delete guard via fill-up existence check

The vehicle delete handler (`src/api/vehicles.rs`) will check `SELECT EXISTS(SELECT 1 FROM fillups WHERE vehicle_id = ?)` before deleting. If fill-ups exist, return `409 Conflict` with `VEHICLE_HAS_FILLUPS`.

**Alternative considered:** CASCADE delete. Rejected because silent data loss is unacceptable for a personal data tracker -- users should explicitly delete fill-ups first or see a clear error.

### 7. Fill-up response shape

```
{
  "id": 1,
  "vehicle_id": 42,
  "date": "2026-04-09",
  "odometer": 15230,
  "fuel_amount": 45.5,
  "fuel_unit": "liters",
  "cost": 72.80,
  "currency": "USD",
  "is_full_tank": true,
  "is_missed": false,
  "station": "Shell Main St",
  "notes": null,
  "created_at": "2026-04-09T10:30:00",
  "updated_at": "2026-04-09T10:30:00"
}
```

Follows the vehicle pattern: database row mapped to response struct via `From`. `date` is TEXT (ISO 8601 date, no time). Booleans default to `false`. Nullable fields: `odometer`, `cost`, `currency`, `station`, `notes`.

### 8. Migration schema

```sql
CREATE TABLE fillups (
    id            INTEGER PRIMARY KEY,
    vehicle_id    INTEGER NOT NULL REFERENCES vehicles(id),
    date          TEXT    NOT NULL,
    odometer      REAL,
    fuel_amount   REAL    NOT NULL,
    fuel_unit     TEXT    NOT NULL DEFAULT 'liters',
    cost          REAL,
    currency      TEXT,
    is_full_tank  INTEGER NOT NULL DEFAULT 0,
    is_missed     INTEGER NOT NULL DEFAULT 0,
    station       TEXT,
    notes         TEXT,
    created_at    TEXT    NOT NULL DEFAULT (datetime('now')),
    updated_at    TEXT    NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_fillups_vehicle_id ON fillups(vehicle_id);
CREATE INDEX idx_fillups_date ON fillups(vehicle_id, date DESC);
```

`odometer` is REAL (not INTEGER) to support fractional readings from trip meters. Booleans are INTEGER (0/1) per SQLite convention. An index on `(vehicle_id, date DESC)` supports the primary list query efficiently.

## Risks / Trade-offs

- **No pagination risk** — A vehicle with thousands of fill-ups would return a large JSON array. Mitigation: typical personal use generates ~50-100 fill-ups/year; pagination can be added later without breaking existing consumers (add `?limit=&offset=` query params).

- **Odometer validation is soft** — Only checks against existing data, not against real-world distance. A user can still enter nonsensical values as long as they're non-decreasing. Mitigation: this catches the most common error (transposed/missing digits); stricter validation would frustrate legitimate edge cases.

- **No FK ON DELETE constraint** — The FK is defined without `ON DELETE CASCADE` or `ON DELETE RESTRICT` because SQLite's FK enforcement depends on `PRAGMA foreign_keys = ON`, which may not be set in all contexts (e.g., migration runner). The application-level check in the vehicle delete handler is the enforced guard. Mitigation: the integration test explicitly covers this path.

- **Unit label is free-text** — `fuel_unit` and `currency` are not validated against a fixed enum at the API level. This keeps the API flexible for international users but risks inconsistent labels. Mitigation: the UI will pre-fill from settings; validation can be tightened later if needed.
