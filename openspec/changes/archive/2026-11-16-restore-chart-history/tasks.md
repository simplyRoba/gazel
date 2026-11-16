# Tasks

## 1. Restore complete chart data and update the API contract

- [x] 1.1 Replace bounded cost/distance preparation in `src/stats/periods.rs` and `src/api/stats/response.rs` with complete monthly/yearly aggregates, including zero-filled gaps in both modes; update helper documentation and verify unit tests cover more than 12 months/years, empty/short histories, leap days, cross-year segments, and direct yearly distance rounding.
- [x] 1.2 Remove fuel-price sampling and price-only limit/wrappers while retaining every eligible observation and the existing full-population trend; verify price tests cover more than 240 observations, date/ID order, equal-date/equal-price observations, zero cost, partial/missed/zero-odometer fills, mixed units, and null/degenerate trends.
- [x] 1.3 Simplify Rust period wire series to `points` and price series to `points`/`trend`, update TypeScript interfaces and affected fixtures in the same change, and remove unused omission/range types; verify `tests/stats.rs`, API-client tests, and type checking agree on removed fields and complete history.
- [x] 1.4 Replace obsolete chart-cap assertions in `src/stats/bounded_tests.rs` and `tests/stats.rs` with full-history coverage while preserving 60-point sparkline tests; verify large histories retain all eligible prices/periods, exact summaries remain unchanged, authentication/errors remain intact, and the shared history-read path is not multiplied by chart or vehicle count.

## 2. Add readable monthly history navigation

- [x] 2.1 Add a shared monthly-window helper/controller using calendar-keyed selection, 12-month steps, clamped overlapping edge windows, and latest-following state; verify focused tests cover 0/1/12/13/25 months, both endpoints, older/newer/prepended/deleted data, and reachability of every month without an empty or isolated oldest window in a long history.
- [x] 2.2 Wire explicit vehicle identity and independent cost/distance window state through `ui/src/routes/+page.svelte`, `ChartsPanel.svelte`, and both period charts, sharing each metric's selection across compact/desktop instances; verify route tests cover selection resets, historical-range preservation on refresh, latest-following refreshes, independent chart windows, and persistence across responsive layouts.
- [x] 2.3 Add localized keyboard/touch-operable previous/next buttons and a concise visible date range to monthly cost/distance charts; show complete yearly series without monthly controls and retain monthly selection across mode changes; verify `PeriodCharts.test.ts` covers navigation/bounds, more than 12 yearly points, exact visible values/units, short histories, and existing cost/distance empty-state thresholds.
- [x] 2.4 Remove `charts.limitedRange` rendering and English/German translations, plus unused caption plumbing only where no other callers remain; verify component tests find no older-period warning, accessible names exist in both locales, and controls/date labels fit compact cards in a responsive visual check.

## 3. Render the complete fuel-price timeline

- [x] 3.1 Remove the reduction caption and `charts.reducedHistory` translations, render every supplied fuel-price point on the full time axis with no window controls, and replace spread-based value extents with an iterative calculation; verify `FuelPriceChart.test.ts` and chart-adapter tests cover more than 240 observations, interior points, first/last dates, actual tooltip values, unchanged trend endpoints, and safe extents on a large array.
- [x] 3.2 Retain existing one-price dot, empty/loading states, response-unit labels, and deterministic same-date observations; add opt-in y-proximity tie-breaking for equal-x fuel-price tooltip candidates and verify mouse/touch tests can inspect different recorded prices on the same date without changing period-chart selection. Verify dashboard tests confirm cost/distance navigation does not alter the fuel-price series or domain.

## 4. Integration and pre-review checks

- [x] 4.1 Extend dashboard/import integration tests with more than 12 years of period data and more than 240 prices; verify old months remain navigable, all years/prices remain supplied, list continuation triggers no statistics request or chart/window changes, navigation triggers no requests, and mutation/import/settings refreshes plus stale-response protection still work.
- [x] 4.2 Validate the change with `npx --yes @fission-ai/openspec validate restore-chart-history --strict` and inspect the final diff; verify the proposal, design, tasks, and three spec deltas agree and no unrequested endpoint/cache/schema changes were introduced.
- [x] 4.3 Before requesting review or archiving, run in order `cargo fmt -- --check`, `cargo clippy -- -D warnings`, `npm run format:check --prefix ui`, `npm run lint --prefix ui`, `npm run check --prefix ui`, and `cargo test`; verify all pass and report any environment blockers without claiming unrun checks passed.

## Workflow follow-up

- Obtain human review before treating the implementation as complete.
- Sync delta specs and archive the change only after implementation, verification, and review; do not rewrite the archived bounded-dashboard-data change.
