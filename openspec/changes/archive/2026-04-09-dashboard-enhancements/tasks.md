## 1. API Client Types and Functions

- [x] 1.1 Add `VehicleStats` interface to `ui/src/lib/api.ts` matching the backend response shape (total_distance, total_fuel, total_cost, fill_up_count, average_efficiency, average_cost_per_distance, distance_unit, volume_unit, currency)
- [x] 1.2 Add `SegmentHistory` interface to `ui/src/lib/api.ts` matching the backend response shape (start_date, end_date, start_odometer, end_odometer, distance, fuel, cost, efficiency, cost_per_distance, is_valid, distance_unit, volume_unit, currency)
- [x] 1.3 Add `fetchVehicleStats(vehicleId)` function to `ui/src/lib/api.ts` — `GET /api/vehicles/{vehicleId}/stats` returning `VehicleStats`
- [x] 1.4 Add `fetchVehicleStatsHistory(vehicleId)` function to `ui/src/lib/api.ts` — `GET /api/vehicles/{vehicleId}/stats/history` returning `SegmentHistory[]`

## 2. Stats Store

- [x] 2.1 Create `ui/src/lib/stores/stats.svelte.ts` with SvelteMap caches for `VehicleStats` and `SegmentHistory[]` keyed by vehicle ID, plus loading and error state
- [x] 2.2 Implement getter functions: `getVehicleStats(vehicleId)`, `getVehicleHistory(vehicleId)`, `getLoading()`, `getError()`
- [x] 2.3 Implement `loadStats(vehicleId)` — fetches both stats and history in parallel, caches results, clears error before call
- [x] 2.4 Implement `loadAllStats(vehicleIds)` — fetches stats for all vehicles in parallel using `Promise.all`
- [x] 2.5 Implement `invalidateStats(vehicleId)` — clears cached data for a vehicle and triggers a refetch

## 3. Dashboard Summary Cards

- [x] 3.1 Add summary card section in `ui/src/routes/+page.svelte` — displays total distance, fill-ups, cost per distance, fuel price (same 4 stats for single or multi-vehicle)
- [x] 3.2 Compute fleet aggregates client-side from per-vehicle stats (sum distance/fuel/cost/fill-ups, derive cost ratios)
- [x] 3.3 Format summary values using `formatDistance`, `formatCurrency` with user settings
- [x] 3.4 Add shimmer/skeleton loading state for summary cards while stats are loading
- [x] 3.5 Handle edge cases: no data shows "—", single vehicle hides chips and per-vehicle row

## 4. Responsive Summary Card Layout

- [x] 4.1 Add CSS Grid layout for summary cards using `auto-fill` / `minmax(~180px, 1fr)` — cards reflow naturally on narrow viewports
- [x] 4.2 Apply design system spacing tokens (gap, padding) and consistent card styling

## 5. Per-Vehicle Stats Display

- [x] 5.1 Show per-vehicle stats below the chip row for the active vehicle: last fill-up date, current efficiency, monthly cost
- [x] 5.2 Format efficiency with `formatEfficiency` and cost with `formatCurrency`
- [x] 5.3 Handle null efficiency (show "—") and loading state (shimmer placeholder)

## 6. Efficiency Badge on Fill-up Cards

- [x] 6.1 Map segment history to fill-up cards by matching `end_date` and `end_odometer` to identify which fill-ups terminate a valid segment
- [x] 6.2 Display efficiency badge on qualifying fill-up cards using `formatEfficiency` with user's units
- [x] 6.3 Style efficiency badge with accent color, consistent with existing badge patterns

## 7. Stats Invalidation on Fill-up Mutations

- [x] 7.1 After successful `createFillup`, `updateFillup`, or `deleteFillup` in the dashboard, call `invalidateStats(vehicleId)` to refetch stats for the affected vehicle
- [x] 7.2 After invalidation, ensure summary cards also update (fleet aggregates recomputed from refreshed per-vehicle data)

## 8. Tests

- [x] 8.1 Vitest tests for stats store: initial state, loadStats success/failure, loadAllStats, invalidateStats, error clearing
- [x] 8.2 Vitest tests for stats display: summary card rendering, per-vehicle stats display, efficiency badge presence/absence
- [x] 8.3 Vitest tests for segment-to-fillup mapping logic (matching end_date/end_odometer)

## 9. Quality Gate

- [x] 9.1 Run `npm run check --prefix ui` (TypeScript type checking)
- [x] 9.2 Run `npm run format:check --prefix ui` and `npm run lint --prefix ui`
- [x] 9.3 Run `cargo fmt -- --check` and `cargo clippy -- -D warnings`
- [x] 9.4 Run `cargo test` (full test suite including UI tests)
