# Proposal

## Why

The dashboard independently reads each vehicle's fill-up history for summaries and segments, transfers unbounded segment arrays, and derives fuel prices from the paginated list. This duplicates work, makes chart data grow with history, and changes the fuel-price chart when older list pages load.

## What Changes

- Use one combined `GET /api/stats?vehicle_id=<id>` request for fleet and vehicle summaries, bounded selected-vehicle charts, and bounded summary sparklines. Keep the statistics API name; do not add `/api/dashboard`.
- Calculate summaries and chart series together from one shared history read per statistics request. Do not add a backend calculation cache, expiry policy, revision tokens, or write-invalidation machinery.
- Keep all-time summary values exact. Limit cost/distance series to the latest 12 months and 12 years, fuel prices to 240 representative recorded prices, and each sparkline to 60 representative values.
- Preserve the first-fill-up fuel-price dot, partial-fill eligibility, existing distance/cost calculations, and chart-card empty states.
- Keep the detailed fill-up list cursor-paginated. Return efficiency metadata only for entries in each requested page, so badges do not require a complete segment-history download.
- Refresh dashboard data after writes and settings changes; prevent stale asynchronous responses from overwriting newer results.
- **BREAKING:** Replace `GET /api/vehicles/{vehicle_id}/stats` and `GET /api/vehicles/{vehicle_id}/stats/history` with the combined statistics endpoint. Remove their handlers, obsolete date-filter contract, and unused client loaders/types rather than retaining a parallel compatibility API. Do not introduce another endpoint that returns every individual price.

## Capabilities

### New Capabilities

None; the existing statistics, fill-up, dashboard, and chart capabilities own these behaviors.

### Modified Capabilities

- `api-vehicle-stats`: One combined statistics endpoint replacing the old summary/history endpoints, explicit chart-size limits, exact all-time aggregates, and fresh request-scoped calculations.
- `api-fillup-crud`: Bounded page-local efficiency metadata alongside the existing list items and cursor.
- `ui-dashboard-stats`: One coordinated data load independent of list pagination, refresh/error handling, and summaries independent of sampled chart data.
- `ui-charts`: Server-prepared bounded series, explicit displayed ranges, sampling semantics, and pagination-stable charts and sparklines.

## Impact

- Backend statistics/calculation code and API routing. No shared backend cache state or cache-invalidation changes to mutation handlers.
- Fill-up page envelopes gain derived metadata; individual fill-up objects and write payloads remain unchanged.
- Frontend API contracts, statistics and fill-up stores, dashboard consumers, chart components, and regression tests.
- No new runtime dependencies, database schema changes, or changes to authentication, odometer validation, or Quick Fill heuristics are planned.
- The discarded unbounded fuel-price-history implementation is not a prerequisite. Build on the committed chart fixes.
