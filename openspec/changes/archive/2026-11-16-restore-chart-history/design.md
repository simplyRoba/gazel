# Design

## Context

See `proposal.md` for motivation. This change crosses the statistics calculation, response contract, and both dashboard chart layouts.

- `src/api/stats.rs` already reads history once per request and coordinates exact fleet/vehicle summaries. Keep that request path.
- `src/api/stats/response.rs` calls `bounded_period_costs`/`bounded_period_distances` and adapts sampled prices into the response.
- `src/stats/periods.rs` also has complete `period_costs`/`period_distances` helpers. They preserve allocation rules but currently zero-fill only monthly gaps; the bounded path zero-fills yearly gaps too.
- `src/stats/prices.rs` extracts eligible prices, samples above 240 observations, and fits a trend to the complete population. `src/stats/sampling.rs` is also used by the summary sparklines and must remain.
- Cost/distance components select the supplied mode and display omission captions. `ChartsPanel.svelte` renders desktop charts; `+page.svelte` also renders compact chart instances directly. Neither chart currently receives vehicle identity.
- The statistics store already rejects stale responses and retains matching data during refresh. These behaviors are not part of the regression to undo.

## Goals / Non-Goals

**Goals:**
- Separate complete returned chart history from the visible monthly window.
- Preserve exact calculation semantics, deterministic ordering, and current refresh boundaries.
- Make navigation local and independent of data/list requests.

**Non-Goals:**
- Backend caching, materialized statistics, migrations, new dependencies, or another endpoint.
- Client-side reconstruction of tank-to-tank segments or cost/distance aggregation.
- Removing summary-sparkline sampling, redesigning chart geometry, adding price navigation, or adding yearly windowing.
- Restoring the removed legacy statistics routes or changing fill-up-page efficiency metadata.

## Decisions

### 1. Complete period aggregates in the combined response

Use the complete aggregation path for both period modes. Zero-fill monthly and yearly gaps from the first applicable period through the last, without padding to the current date or before the first applicable period. Cost spans come from segment ending periods; distance spans include each full segment interval. Reuse calendar arithmetic and direct yearly allocation; never aggregate rounded monthly distance values to produce yearly totals.

Represent each period mode as `{ points: [{ period, value }] }`. Remove `PERIOD_LIMIT`, the bounded-only implementation, and period omission/range/count metadata where no longer used. Monthly UI navigation can derive its range from ordered period labels.

**Alternative rejected:** date-window API parameters and a new request for every arrow click. All history is already read for exact summaries; complete period aggregates allow navigation without another read, network lifecycle, or stale-response path. This intentionally gives up fixed chart response budgets, not the shared-read improvement.

### 2. Monthly windows are presentation state

Use a small shared window helper/controller for cost and distance rather than duplicating calendar/index logic. The ordered, gap-filled monthly aggregate array is the source of truth. A view contains up to 12 consecutive months:

- Initially select the latest window, ending at that metric's latest data month.
- Move by 12 months with previous/next controls, clamping at the earliest and latest valid starts. An edge window may overlap its neighbor so the oldest month is not isolated as a one-point distance view.
- Hide navigation when the source has at most 12 months. Otherwise show localized accessible previous/next buttons, disabled at the respective endpoint, and a short locale-formatted date range. Do not add explanatory omission text.
- Yearly mode renders every supplied yearly point and hides monthly navigation. Preserve the selected monthly range while switching modes.
- Keep cost and distance window selections independent. A cost-window action must not move distance or fuel-price history.

Retain selection by calendar label, not only an array index: new older data can prepend periods during refresh. Track whether the view is following the latest window. On a same-vehicle refresh, latest-following views remain latest; historical views keep their calendar range if possible and clamp when periods disappear. On vehicle change, reset to the new vehicle's latest window.

Expose explicit vehicle identity through the chart boundary rather than guessing identity from a new response object. Own the per-metric window state at the dashboard boundary and share it with compact and desktop instances, so a viewport change does not reset the chosen dates. Keep this presentation state separate from the API store's request counter and response state.

**Alternatives rejected:** putting every monthly bar on screen would crowd long histories; showing only the latest window without navigation recreates the regression. No yearly window is needed for this change: yearly mode remains a full-history overview.

### 3. Complete recorded prices, unchanged full-population trend

Keep existing eligibility and normalization in `fuel_prices`, including zero-cost, partial/missed, and zero-odometer fills. Return all eligible prices in date-then-ID order without sampling or deduplicating equal dates/values. Fit the trend to that same complete population with the existing calculation; retain `null` for fewer than two observations or a zero-width date domain.

The fuel-price wire shape becomes `{ points: [{ date, value }], trend: [...] | null }`. Remove price-only sampling wrappers, the 240-point constant, and reduction/range/count metadata. Keep shared representative sampling and the 60-point sparkline limit intact.

`FuelPriceChart.svelte` renders the complete supplied array on one continuous time axis. Remove its reduction caption. Replace spread-based `Math.min(...values)`/`Math.max(...values)` with an iterative extent calculation to avoid argument-limit failures when full history is large; this does not discard observations.

The shared `Tooltip.svelte` currently chooses the first observation at the nearest x-coordinate. For fuel prices, resolve equal-x candidates by proximity to the pointer's y-coordinate so different prices on the same date remain inspectable by hover and touch. Make that tie-breaking opt-in to avoid changing cost/distance behavior. Identical date/value observations may overlap visually but must remain separate in the series.

**Alternative rejected:** retaining sampling but hiding its caption would conceal missing observations, not restore history.

### 4. Update the contract and remove obsolete narration together

Update Rust serialization, TypeScript interfaces in `ui/src/lib/api.ts`, adapters, and all affected test fixtures in the same implementation. Keep `points`, `trend`, selected vehicle identity, response units, and sparkline contracts stable. Remove unused period/price range types after checking references.

Remove `charts.limitedRange` and `charts.reducedHistory` from English and German translations and their rendering paths. Keep empty-state and loading/error text. Remove generic caption support only if it has no remaining callers; the navigation range is a control label, not a replacement warning paragraph.

Keep existing coordinated refreshes, matching-response units, stale-result protection, and list-pagination independence. Neither mode switches nor window navigation should call `/api/stats` or load another fill-up page.

## Risks / Trade-offs

- **Price payload/rendering grows with eligible fills; period payloads grow with calendar span** → return compact numeric series, calculate extents without argument spreading, and exercise large histories. Do not silently restore a cap. There is no measured latency claim behind this change; any later performance proposal needs measurements and must preserve access.
- **Complete allocation can visit more periods than the latest-window implementation** → reuse complete calendar helpers and test long cross-period intervals. Preserve calculation error handling rather than introducing date cutoffs.
- **Yearly zero-fill differs from the existing complete helper's sparse output** → explicitly test gaps and preserve the currently observable zero-filled calendar behavior while extending it to the full span.
- **Window state can drift after edits, imports, vehicle changes, or responsive layout changes** → calendar-keyed selection, explicit vehicle identity, shared presentation state, and route-level regression coverage.
- **Full prices include same-date observations** → preserve every observation and deterministic ordering; do not deduplicate to simplify rendering or tooltips.
- **Removed response fields break consumers** → document the exact removals in the proposal/spec migration notes and deploy the embedded UI with the backend.

## Migration Plan

1. Update full-history calculations, response types, and focused Rust tests.
2. Update the UI contract/fixtures, monthly window controls/state, fuel-price rendering, and translations with focused UI/route tests.
3. Run strict change validation and the repository pre-review checks. Review the change before treating it as complete or archiving it; canonical specs are synced through the normal review/archive workflow, not during planning.
4. Deploy backend and embedded UI together. No stored-data migration is required. A binary rollback restores the old contract and its history restrictions without modifying data.
