## 1. Module setup and types

- [x] 1.1 Create `src/api/stats.rs` with module declaration and add `pub mod stats;` to `src/api/mod.rs`
- [x] 1.2 Define `StatsRow` (sqlx `FromRow`) for the fillup fields needed by calculations: `id`, `date`, `odometer`, `fuel_amount`, `fuel_unit`, `cost`, `is_full_tank`, `is_missed`
- [x] 1.3 Define `StatsQuery` (Deserialize) for the `from`/`to` query parameters with validation (return `STATS_INVALID_DATE_FILTER` on bad format)
- [x] 1.4 Define `VehicleStats` response struct (Serialize) with fields: `total_distance`, `total_fuel`, `total_cost`, `fill_up_count`, `average_efficiency` (Option), `average_cost_per_distance` (Option), `distance_unit`, `volume_unit`, `currency`
- [x] 1.5 Define `SegmentHistory` response struct (Serialize) with fields: `start_date`, `end_date`, `start_odometer`, `end_odometer`, `distance`, `fuel`, `cost`, `efficiency`, `cost_per_distance`, `is_valid`, `distance_unit`, `volume_unit`, `currency`

## 2. Calculation engine

- [x] 2.1 Implement `fetch_fillups_for_stats()` -- query fillups ordered by `date ASC, id ASC`, filtered by optional `from`/`to`, excluding rows with `odometer <= 0`
- [x] 2.2 Implement unit normalization helpers: `normalize_fuel_to_liters(amount, fuel_unit)` and `convert_distance(value, from, to)` / `convert_volume(value, from, to)` using the conversion factors from the design doc
- [x] 2.3 Implement `compute_segments()` -- walk fillups in order, identify full-tank boundaries, accumulate fuel/cost per segment, flag segments containing missed fill-ups as invalid. Return `Vec<Segment>` (internal struct with raw computed values)
- [x] 2.4 Implement `aggregate_stats()` -- from the list of segments, compute totals (distance, fuel, cost, fill-up count) and averages (efficiency, cost-per-distance using only valid segments)

## 3. Handlers and routing

- [x] 3.1 Make `ensure_vehicle_exists` in `fillups.rs` accessible from stats module (change visibility to `pub(crate)`)
- [x] 3.2 Implement `summary` handler (`GET /api/vehicles/{vehicle_id}/stats`) -- validate vehicle exists, parse query params, fetch fillups, compute segments, aggregate, read settings for unit conversion, return `VehicleStats`
- [x] 3.3 Implement `history` handler (`GET /api/vehicles/{vehicle_id}/stats/history`) -- same pipeline but return `Vec<SegmentHistory>` with per-segment data points
- [x] 3.4 Register routes in `src/api/mod.rs`: `.route("/vehicles/{vehicle_id}/stats", get(stats::summary))` and `.route("/vehicles/{vehicle_id}/stats/history", get(stats::history))`
- [x] 3.5 Add error codes to `default_message()` in `error.rs`: `STATS_INVALID_DATE_FILTER`

## 4. Integration tests

- [x] 4.1 Create `tests/stats.rs` with test helpers for seeding vehicles and fillups
- [x] 4.2 Test: summary endpoint with no fill-ups returns zeroes and null averages
- [x] 4.3 Test: summary endpoint with a single fill-up returns totals but null efficiency
- [x] 4.4 Test: summary endpoint with multiple full-tank fills returns correct averages
- [x] 4.5 Test: summary endpoint with missed fill-up segments excludes them from efficiency average
- [x] 4.6 Test: summary endpoint with partial-tank fills correctly accumulates fuel across segments
- [x] 4.7 Test: history endpoint returns correct segment array with `is_valid` flags
- [x] 4.8 Test: history endpoint with fewer than 2 full-tank fills returns empty array
- [x] 4.9 Test: time-range filtering (`from`/`to` query params) restricts which fillups are included
- [x] 4.10 Test: invalid date filter returns 400 with `STATS_INVALID_DATE_FILTER`
- [x] 4.11 Test: non-existent vehicle returns 404 with `VEHICLE_NOT_FOUND`
- [x] 4.12 Test: unit conversion -- stats response reflects current settings (metric vs imperial)

## 5. Lint, format, and verify

- [x] 5.1 Run `cargo fmt -- --check` and fix any formatting issues
- [x] 5.2 Run `cargo clippy -- -D warnings` and resolve all lints
- [x] 5.3 Run `cargo test` and confirm all tests pass (existing + new)
