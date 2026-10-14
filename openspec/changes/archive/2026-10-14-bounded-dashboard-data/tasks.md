# Tasks

## 1. Shared statistics calculations

- [x] 1.1 Extract reusable tank-to-tank and summary helpers from `src/api/stats.rs`, retaining identity and chronology; preserve existing numeric fixtures and same-day ordering expectations as calculation-level regressions independent of the old HTTP routes.
- [x] 1.2 Implement fleet and displayed-average aggregation from complete eligible populations; test unequal vehicle segment counts, rounding order, invalid segments, zero/null odometers, and fleet versus active cost ratios.
- [x] 1.3 Port period cost/distance calculations to the backend using inclusive calendar days and existing rounding; test month/year boundaries, leap days, zero-filled gaps, and clock-change dates against the delta requirements.
- [x] 1.4 Add individual-price calculation with mixed stored fuel units and existing eligibility; test first partial/missed fills, missing/zero odometers, free fuel, invalid numbers, same-day ordering, and unrounded prices.

## 2. Bounded chart series

- [x] 2.1 Implement latest-12-month and latest-12-year windows with range/omission metadata; test long histories, missing periods, and unchanged exact all-time totals.
- [x] 2.2 Implement deterministic min/max sampling for 240 prices and 60 sparkline values, documenting endpoint/extrema retention and original positions; test limits, small populations, constant values, same-date ties, and deterministic output.
- [x] 2.3 Compute fuel-price regression from the full eligible population before sampling and return only its endpoints; test that sampling does not bias the trend and that one-point/zero-width inputs have no trend.

## 3. Replace the statistics endpoints

- [x] 3.1 Replace the old contracts with typed `GET /api/stats` query/response types and safe selection-error translations; verify empty fleet, valid/missing/invalid selections, units, and authentication boundaries with endpoint tests.
- [x] 3.2 Compute all statistics from one lightweight chronological fill-up-history query in a coherent read transaction per request; instrument reads and verify exactly one shared history read per request, selected-only chart series, and no raw history arrays in the response.
- [x] 3.3 Keep responses uncached and send `Cache-Control: no-store`; verify a subsequent request reflects committed fill-up, vehicle, settings, import, and external database changes without backend invalidation hooks or an expiry delay, and calculation failures use the normal safe error response.
- [x] 3.4 Remove old per-vehicle summary/history routes and handlers, unused full-history response types, and obsolete date-filter parsing; verify old paths no longer supply statistics responses, no `/api/dashboard` or compatibility alias exists, and calculation regressions still pass.
- [x] 3.5 Document response limits, exact all-time summaries, request-scoped calculation, and the breaking replacement of old routes/date filters; verify the documented contract against endpoint tests and release-note migration guidance.

## 4. Page-local efficiency badges

- [x] 4.1 Add `FillupPage.efficiency` metadata using targeted page-boundary/interval aggregation; verify metadata count equals page item count and pagination does not invoke a full statistics calculation.
- [x] 4.2 Test a starting full tank outside the page, intervening partial/missed fills, same-day identities, empty pages, mixed units, and all existing cursor/limit cases; verify individual GET/POST/PUT fill-up shapes remain unchanged.
- [x] 4.3 Extend frontend page types and paginated store chains to retain ID-keyed efficiency metadata with response units; test deduplication, continuation errors, page resets, unit changes, and superseded requests.

## 5. Dashboard and chart consumers

- [x] 5.1 Add one typed `/api/stats` request and coordinated response state with a frontend request counter; test one request per load, vehicle selection, failed-refresh retention, state resets, and reversed response completion.
- [x] 5.2 Replace consumers of complete segment arrays, browser period aggregation, and paginated fuel-price data with the bounded response; test both layouts, existing thresholds, first-price dots, mode switching, response units, and limited/reduced range labels.
- [x] 5.3 Read exact summary values and bounded sparklines from the response and use page metadata for badges; test summaries under heavy sampling, preserved sparkline positions, and badge independence from chart samples.
- [x] 5.4 Refresh displayed statistics after dashboard/global-CTA fill-up mutations, vehicle changes, settings changes, and import completion; test the refresh calls without changing Quick Fill's loaded-list heuristics or adding backend cache invalidation.
- [x] 5.5 Add a scrolling regression with more history than a list page and both sampling budgets: load continuation pages and assert unchanged chart paths, axes, values, trend endpoints, summaries, and sparklines, with no additional statistics/history requests.
- [x] 5.6 Remove unused per-vehicle API loaders, `loadAllStats` loops, full-history getters/types, and obsolete endpoint tests; verify call-site searches and UI tests show no removed-route fallback.

## 6. Integration and review gate

- [x] 6.1 Verify large-history response bounds, one shared history read per statistics request, removal of superseded routes/loaders, and a production UI build; record before/after calculation timings and confirm browser network traces contain no full-history fallback or background list traversal.
- [x] 6.2 Run `cargo fmt -- --check`, `cargo clippy -- -D warnings`, `npm run format:check --prefix ui`, `npm run lint --prefix ui`, `npm run check --prefix ui`, and `cargo test`; require all checks to pass before human review or archiving.

## Workflow follow-up

- Obtain human review of the proposed limits and implementation before archiving.
- Sync delta specs and archive only after the bounded data flow and regression checks are complete.
