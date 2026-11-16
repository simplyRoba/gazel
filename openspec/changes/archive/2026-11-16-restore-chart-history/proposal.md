# Proposal

## Why

Dashboard cost and distance charts discard periods older than their latest 12 months or years, with no way to reach them. Fuel-price charts also omit recorded observations above 240 points; restore access to the full history without squeezing every month into a single chart.

## What Changes

- Return complete monthly and yearly cost/distance aggregates through the existing `/api/stats` response.
- Show at most 12 calendar months at a time in cost/distance monthly mode, initially the latest window, with previous/next navigation through the entire history. Yearly mode shows the complete yearly series without a 12-year cutoff.
- Show the full fuel-price timeline with every eligible recorded price, including histories above 240 observations. Do not add period navigation to this chart.
- Remove older-period omission and fuel-price reduction messages. Use a concise visible date range with monthly navigation instead of explanatory chart narration.
- Keep exact summaries, existing segment and unit rules, bounded summary sparklines, the shared history read, and independence from fill-up-list pagination.
- **BREAKING**: remove obsolete period-omission and price-reduction metadata from `/api/stats` chart series. Keep the endpoint and its chart `points`/price `trend` structure; chart payloads now grow with historical periods and eligible recorded prices.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `api-vehicle-stats`: return complete period aggregates and recorded fuel prices instead of capped chart series; retain bounded sparklines and exact summaries.
- `ui-charts`: navigate monthly cost/distance windows, show all years and recorded fuel prices, and remove omission/reduction captions.
- `ui-dashboard-stats`: load complete chart history in the coordinated response while preserving refresh, selection, and pagination behavior.

## Impact

- Backend: `src/stats/periods.rs`, `src/stats/prices.rs`, chart response adapters in `src/api/stats/response.rs`, related exports and tests.
- Frontend: chart API types, cost/distance navigation and selection state, fuel-price rendering, English/German translations, and affected fixtures/tests.
- Existing response consumers must stop relying on reduction metadata or fixed chart-point limits. Backend and embedded UI deploy together; no database migration, new dependency, cache, or separate history endpoint is needed.
- Canonical specs currently require the limits being removed; this change explicitly replaces those requirements rather than rolling back unrelated dashboard fixes.
