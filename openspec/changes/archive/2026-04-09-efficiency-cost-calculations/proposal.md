## Why

The app stores all raw fill-up data (odometer, fuel amount, cost, full-tank/missed flags) but has no engine to compute the metrics users actually care about: fuel efficiency, cost per distance, and aggregate spending. The UI already has `formatEfficiency` ready to display values, but nothing produces them. This is the next chunk on the roadmap (chunk 9) and unblocks dashboard enhancements (chunk 10) and charting (chunk 11).

## What Changes

- Add a calculation engine that derives fuel efficiency (distance / fuel) between consecutive full-tank fill-ups, gracefully handling missed fill-ups and partial tanks
- Add cost-per-distance calculation using the same segment logic
- Add aggregation queries: average efficiency, total cost, total fuel, total distance -- per vehicle, filterable by time range (month, year, all-time)
- New `GET /api/vehicles/:id/stats` endpoint returning vehicle summary stats
- New `GET /api/vehicles/:id/stats/history` endpoint returning time-series efficiency/cost data for charting
- All stats respect the user's configured unit system (distance_unit, volume_unit, currency)
- Integration tests covering calculation correctness: single fill-up, missed fill-ups, partial tanks, unit conversions

## Capabilities

### New Capabilities

- `api/vehicle-stats`: Backend calculation engine and REST API for fuel efficiency, cost-per-distance, and aggregate vehicle statistics. Covers the stats and stats/history endpoints, segment calculation logic, time-range filtering, and unit-system-aware responses.

### Modified Capabilities

_(none -- this change adds new endpoints without altering existing fillup or settings behavior)_

## Impact

- **New code**: `src/api/stats.rs` (handlers + calculation logic), route registration in `src/api/mod.rs`
- **New test file**: `tests/stats.rs` with integration tests for both endpoints and calculation edge cases
- **Existing code touched**: `src/api/mod.rs` (register new routes), possibly `src/api/fillups.rs` (reuse query helpers or the `FillupRow` type)
- **Dependencies on existing specs**: `api/fillup-crud` (data model, ordering guarantees), `api/settings` (unit system for response formatting)
- **No schema migrations**: stats are computed from existing fillup data, not stored
- **No frontend changes in this chunk**: UI consumption happens in chunks 10-11
