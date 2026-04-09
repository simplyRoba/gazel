## Context

The app tracks fuel fill-ups with odometer readings, fuel amounts, costs, and flags for full-tank/missed fills. All raw data is present to compute fuel efficiency and cost metrics, but no calculation logic or stats endpoints exist. The UI already ships `formatEfficiency()` (from the unit-formatting spec) but has nothing to feed it.

Fillups are stored with `odometer`, `fuel_amount`, `cost`, `is_full_tank`, and `is_missed` fields. Settings provide `distance_unit` (`km`/`mi`) and `volume_unit` (`l`/`gal`) which determine how stats should be presented. The existing handler pattern uses `SqlitePool` extraction, `FillupRow`/`Fillup` type pairs, shared SQL `const`s, and `ApiError` for all error paths.

## Goals / Non-Goals

**Goals:**

- Compute per-segment fuel efficiency (distance / fuel between consecutive full-tank fill-ups)
- Compute per-segment cost-per-distance
- Aggregate stats per vehicle: average efficiency, total cost, total fuel, total distance
- Support time-range filtering (month, year, all-time)
- Provide a time-series history endpoint for downstream charting (chunk 11)
- Return stats in the user's configured unit system
- Handle edge cases gracefully: zero/one fill-ups, missed fill-ups, partial tanks

**Non-Goals:**

- Frontend UI for stats (chunks 10-11)
- Caching or materialized views for stats (premature for SQLite single-user scale)
- Cross-vehicle fleet aggregation (per-vehicle only in this chunk)
- Fuel price normalization or currency conversion between historical rates

## Decisions

### 1. Compute on read, not on write

**Decision:** Calculate stats on every request by querying fillups and computing in Rust, rather than storing pre-computed values in a stats table.

**Rationale:** The dataset per vehicle is small (hundreds of fill-ups at most for a personal tracker). SQLite reads are fast, and computing in Rust avoids a secondary data store that must be kept in sync when fill-ups are created/updated/deleted. No migration needed.

**Alternative considered:** Materialized stats table updated via triggers or on fillup mutation. Rejected because it adds schema complexity, migration burden, and consistency risk for negligible performance gain at this scale.

### 2. Segment-based efficiency algorithm

**Decision:** Walk fillups in ascending odometer order. A valid efficiency segment starts and ends at a full-tank fill-up with no missed fill-ups in between. Accumulate fuel and cost across intermediate partial-tank fills within the segment.

Algorithm:
1. Query all fillups for the vehicle, ordered by `date ASC, id ASC`.
2. Find pairs of consecutive full-tank fills (`is_full_tank = true`).
3. Between each pair, sum `fuel_amount` and `cost` of all fills (including the ending full-tank fill, excluding the starting one).
4. If any fill in the segment has `is_missed = true`, mark the segment as invalid (skip it for efficiency, still count fuel/cost for aggregates).
5. Efficiency = `(end_odometer - start_odometer) / sum_fuel`. Cost per distance = `sum_cost / (end_odometer - start_odometer)`.

**Rationale:** This is the standard "tank-to-tank" method used by fuel tracking apps (Fuelly, Spritmonitor). It handles partial fills correctly by accumulating fuel until the next full-tank event gives a reliable distance reference point. Skipping missed-fill segments avoids reporting inaccurate efficiency (the distance gap doesn't match the fuel recorded).

**Alternative considered:** Simple consecutive-pair calculation (every two adjacent fills). Rejected because partial tanks produce wrong efficiency values.

### 3. Unit conversion in the response layer, not the calculation layer

**Decision:** Perform all internal calculations in the raw stored units (the units attached to each fillup record, which match the settings at time of recording). Convert to display units only when building the API response.

**Rationale:** Fill-ups store `fuel_unit` and the raw `odometer` value. If a user changes their unit preference from metric to imperial, historical data doesn't change -- the stats endpoint reads the current settings and converts the computed raw values. This keeps calculation logic unit-agnostic and avoids precision loss from double-conversion.

**Conversion factors:**
- Distance: 1 mi = 1.609344 km
- Volume: 1 gal = 3.785411784 L
- Efficiency is derived: distance_unit / volume_unit (e.g., km/L, mi/gal)

### 4. Two endpoints, not one

**Decision:** Separate summary endpoint (`/stats`) from history endpoint (`/stats/history`).

- `GET /api/vehicles/:id/stats` -- single JSON object with aggregate totals and averages.
- `GET /api/vehicles/:id/stats/history` -- array of per-segment data points for charting.

**Rationale:** The summary endpoint is cheap (just aggregates) and serves the dashboard. The history endpoint returns a larger payload (one entry per segment) and is only needed for charts. Separating them lets chunk 10 (dashboard) call only the lightweight summary endpoint while chunk 11 (charts) uses the history endpoint.

**Alternative considered:** Single endpoint with a `?detail=full` query param. Rejected because the response shapes are materially different (single object vs. array), making a combined response awkward for typed frontends.

### 5. Time-range filtering via query parameters

**Decision:** Both endpoints accept optional `from` and `to` query parameters (ISO date strings) to filter fillups by date before computing stats.

**Rationale:** Enables month/year/all-time views without separate endpoints. If omitted, all fillups are included (all-time). The filter applies at the SQL query level for efficiency.

### 6. New module `src/api/stats.rs`

**Decision:** All stats logic lives in a single new module. Reuse `ensure_vehicle_exists` from fillups (make it `pub(crate)` or extract to a shared helper). Define a lightweight `StatsRow` for the SQL query and dedicated response types.

**Rationale:** Follows the existing pattern of one module per resource. The calculation engine is tightly coupled to the handler (no other consumers yet), so keeping it co-located is simpler than a separate `src/engine/` module.

Route registration in `mod.rs`:
```
.route("/vehicles/{vehicle_id}/stats", get(stats::summary))
.route("/vehicles/{vehicle_id}/stats/history", get(stats::history))
```

## Risks / Trade-offs

**[Risk] Fillups with missing odometer values** -- Legacy rows may have `NULL` odometer (mapped to `0.0` in `Fillup`). Segments with zero odometer produce meaningless efficiency.
→ *Mitigation:* Skip fillups with `odometer <= 0.0` when building segments. Document that stats require odometer data.

**[Risk] Mixed unit fill-ups** -- If a user changes their volume_unit setting between fill-ups, historical records have `fuel_unit` values that differ from the current setting.
→ *Mitigation:* When computing stats, normalize all fuel amounts to a common unit (liters) before summing, then convert the final result to the user's current display unit. Same approach for distance if odometer units ever vary.

**[Risk] Performance on large datasets** -- Computing stats from raw fillups on every request.
→ *Mitigation:* Acceptable at personal-tracker scale (< 1000 fillups per vehicle). If needed later, add a simple in-memory LRU cache keyed by `(vehicle_id, from, to)` with invalidation on fillup mutations. Not implementing this now.

**[Trade-off] No pagination on history** -- The history endpoint returns all segments. For a personal tracker this is fine (a few hundred segments at most), but wouldn't scale for fleet management.
→ *Accepted:* Out of scope per non-goals.
