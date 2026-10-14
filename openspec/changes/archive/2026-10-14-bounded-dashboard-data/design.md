# Design

## Context

See `proposal.md` for the problem and scope. Relevant existing boundaries:

- `src/api/stats.rs` independently fetches the same history for summary and segment-history requests. Its tank-to-tank engine, rounding, and conversion rules are the calculation baseline.
- `ui/src/lib/stores/stats.svelte.ts` makes two requests per vehicle; the dashboard builds fleet averages and sparklines from complete segment arrays.
- Cost and distance components aggregate segments in the browser. Monthly mode slices to 12 months; yearly mode is currently unlimited.
- Fuel price uses the loaded fill-up chain. Loading a continuation page changes its points and axes.
- `buildEfficiencyMap` is the remaining non-chart consumer of complete segment history. Its date/odometer key can assign badges to the wrong same-day entry and conflicts with converted odometers.
- Quick Fill and detailed forms use loaded fill-ups, not the statistics history. Their helpers are outside this change.
- `AppState` currently holds the SQLite pool and authentication state. There is no backend statistics cache. SQLite uses WAL and pooled connections.

## Goals / Non-Goals

**Goals:**
- One shared fill-up-history read per statistics request, not one scan per chart or statistics endpoint.
- Response size proportional to vehicle count and fixed chart budgets, not fill-up count.
- No chart dependency on list pagination or unbounded browser-side segment maps.
- Exact all-time displayed metrics, independently calculated from reduced visualizations.
- Fresh, coherent calculations from the database without backend cache infrastructure.

**Non-Goals:**
- Backend result caching, expiry, revision tokens, write-invalidation plumbing, request coalescing, or incremental/materialized database statistics.
- Maintaining compatibility aliases for the replaced per-vehicle summary/history APIs.
- Fetching every list page in the background or adding a complete individual-price-history endpoint.
- Redefining efficiency, cost allocation, currency conversion, odometer validation, or Quick Fill heuristics.
- New dependencies or database migrations.

## Decisions

### 1. One request-scoped statistics calculation

Use `GET /api/stats?vehicle_id=<id>`. Omit the parameter for fleet-only data. Keep the statistics naming and replace the old per-vehicle summary/history routes rather than introducing `/api/dashboard` or retaining compatibility aliases. Fetch vehicles, settings, and a lightweight chronological fill-up stream in one SQLite read transaction per request. The fill-up query is ordered by vehicle ID, date, and fill-up ID. Read history once and reuse that input across calculations; do not compose the old HTTP handlers, which would repeat their queries.

Build exact per-vehicle summaries, fleet sums/counts, and the bounded fleet efficiency trend from that input. Prepare detailed chart series only for the selected vehicle. Return aggregates and reduced series, never raw rows or complete segment vectors. A vehicle selection starts a new statistics request with one shared history read; pagination starts no statistics request.

Proposed response groups:

| Group | Contents | Bound |
| --- | --- | --- |
| `units` | Distance, volume, currency used for this response | Constant |
| `fleet` | Existing displayed totals/ratios and exact average efficiency | Constant |
| `vehicles` | Numeric summaries keyed by vehicle ID | One summary per vehicle |
| `fleet_efficiency` | Representative sparkline values, original positions, source count | 60 values |
| `selected` | Selected ID, summary, charts, cost-per-distance sparkline; otherwise `null` | One vehicle |
| `selected.charts.cost` | Monthly and yearly `{period, value}` arrays with range metadata | 12 + 12 periods |
| `selected.charts.distance` | Monthly and yearly `{period, value}` arrays with range metadata | 12 + 12 periods |
| `selected.charts.fuel_price` | `{date, value}` retained recorded prices, source count, reduction/range metadata, optional two-point fitted trend | 240 prices + 2 trend endpoints |
| `selected.cost_per_distance` | Representative sparkline values and original positions | 60 values |

Unit/currency metadata belongs to the response, not every point. No station, notes, raw fuel, odometer, or segment interval records are required for charts.

**Alternative rejected:** another channel returning one compact record for every fill-up. Reducing fields does not bound entry count and does not solve the underlying problem.

### 2. Preserve calculation populations before reducing charts

Extract reusable pure calculation helpers from `stats.rs`, preserving calculation semantics but not the old wire contracts. Remove the superseded summary/history handlers, route registrations, obsolete date-filter parsing, and unused full-history response types once the combined statistics endpoint is wired.

The shared query must include zero/null-odometer rows because they can have valid fuel prices. Apply the current statistics-row eligibility separately for existing totals and segment calculations. Do not accidentally apply the odometer filter to fuel prices.

Preserve these distinctions:

- Displayed fleet and active efficiency average the eligible, converted, rounded segment values used by the current UI. Weight segments equally, not vehicles equally.
- Fleet cost per distance remains summed vehicle total cost divided by summed vehicle total distance. Active segment-derived ratios retain their existing calculation.
- Cost/distance series include segments whose efficiency is invalid. Efficiency averages and efficiency sparklines exclude them.
- Segment costs belong to the ending period; distances are apportioned by inclusive calendar days and period totals retain integer rounding.
- Use `NaiveDate` calendar arithmetic for allocation; do not reproduce timezone/DST artifacts from elapsed local JavaScript milliseconds.
- Normalize each individual fill's fuel before dividing cost by volume. Do not round price values before sampling or trend fitting. Preserve zero-cost fills and partial/missed fills.

All-time sums/counts and complete-population trend fits are calculated before any series reduction. Formatting remains on the frontend. Currency labels follow existing settings behavior; this does not add foreign-exchange conversion.

### 3. Fixed server-side visual budgets

These are the proposed initial bounds, enforced by the API rather than optional client slicing:

| Series | Budget | Rule |
| --- | --- | --- |
| Monthly cost/distance | 12 months | Window ends at latest represented data month; zero-fill only inside that window |
| Yearly cost/distance | 12 years | Equivalent calendar-year window; explicitly signal omitted older periods |
| Fuel price | 240 recorded prices | All-time representative subset; first/last and extrema retained |
| Each sparkline | 60 values | All-time representative subset with original relative positions |

For fuel price, return all values when count is at most 240. Otherwise preserve the first and last values, partition the chronological interior into 119 contiguous index buckets, retain the minimum and maximum from each bucket, deduplicate, and sort retained original indices. Resolve equal extrema deterministically by earliest original index. This yields at most `2 + 119 * 2 = 240` actual recorded prices without inventing averages.

Use the same rule with 29 interior buckets for each 60-value sparkline. Preserve original normalized index positions rather than renumbering sampled points, which would distort the existing index-based trend shape. Sort fleet efficiency inputs by segment end date and a deterministic vehicle/fill-up identity tie-breaker.

Compute fuel-price regression from the full eligible price population and send its two endpoints; omit it for fewer than two points or a zero-width date domain. Rendering a regression of the retained extrema would bias the displayed trend.

Expose source count, displayed range, and reduction/omission status. Use concise chart labels to distinguish a latest-period window or reduced history from a complete detailed history. Price tooltips identify retained real prices. Detailed history remains available through the paginated list and existing export API.

### 4. No backend calculation cache

Compute each statistics response from its own consistent database read view. Use the existing pool extraction; `AppState` does not need cache fields, generations, clocks, or locks. Backend mutation handlers need no cache-invalidation changes. A statistics request begun after an application or external database commit sees that committed data naturally.

Return `Cache-Control: no-store` for statistics responses so HTTP caching does not introduce another freshness policy. Calculation failures use the normal safe API error response; the server does not substitute an older computed result.

**Alternative rejected for now:** backend result caching. It adds expiry, invalidation across write paths, and concurrent rebuild handling without measurements showing a need. First measure the one-read calculation; caching can be proposed separately if it remains a bottleneck.

### 5. Page-local badges without full history arrays

Extend only the list-page envelope:

```text
{ items, next_cursor,
  efficiency: { distance_unit, volume_unit,
    values: [{ fillup_id, efficiency: number | null }] } }
```

Keep individual GET/POST/PUT fill-up objects and write bodies unchanged. Store the page metadata alongside each paginated chain and associate badges by fill-up ID, not date/converted odometer.

For the requested page, batch lookup only the segment contexts necessary for its full-tank endings: their previous eligible full-tank boundaries and fuel/missed-fill aggregates across those bounded chronological intervals. Use existing date-then-ID ordering and shared conversion rules. An interval may begin outside the page; computing from only the returned 25 rows is incorrect.

Do not call the full dashboard builder or fetch all segment history to annotate each continuation page. These targeted context queries are separate from dashboard calculations and return at most one scalar per page item. A very long partial-fill interval can still require an interval aggregate in SQLite, but it must not become a full-history download or one fleet scan per page.

### 6. Coordinated frontend response lifecycle

Replace `loadAllStats`, per-vehicle summary/history API loaders, complete-history getters, browser period aggregation, and `getFillups()` fuel-price derivation with one typed `/api/stats` request in the existing statistics-store boundary. Remove obsolete client loaders/types instead of retaining an old-endpoint fallback. Retain the currently displayed response for loading/error handling, not as a backend calculation cache. Each mount, vehicle selection, or relevant refresh requests fresh statistics. Both responsive layouts consume the same response; modes use the supplied monthly/yearly bundles without another history download.

Read summary values directly from the response. Keep the existing card/point thresholds, first-price dot, loading skeletons, and chart geometry components. Supply bounded sparkline values directly and page-local efficiency metadata for badges. Quick Fill's loaded-list helpers remain unchanged.

Create/edit/delete, the global CTA, vehicle changes, settings changes, and import completion must refresh the displayed statistics. Clear page metadata when unit settings change. Retained values keep their response units until replacement data arrives. Only retain selected-vehicle charts for the matching vehicle.

Use a frontend request counter so an old response cannot overwrite a completed refresh, change another vehicle's chart, or repopulate reset state. This is normal asynchronous UI state handling, not backend cache coordination. Pagination neither clears the displayed statistics nor sends a statistics request.

## Risks / Trade-offs

- **Each statistics calculation still costs O(fill-up count)** → perform one shared history read, calculate once per request, and keep browser responses bounded. Measure before proposing caching or incremental statistics.
- **Sampling removes some detailed tooltip targets** → retain real values, endpoints and extrema; label reduced history; keep detailed list/export access. Exact summaries and trend fitting use the full population.
- **Yearly mode now omits years older than its latest 12** → expose the displayed range and omission metadata rather than imply all-time bars.
- **Porting calculations can subtly change metrics or badges** → compare backend outputs with existing fixtures, mixed-unit rounding, unequal fleet segment counts, calendar transitions, and same-day identities. Preserve intended badge association rather than the existing key-collision bug.
- **A missed frontend refresh leaves old displayed values** → test refresh calls after relevant writes and reject out-of-order UI responses. The next statistics request always reads current database contents.

## Migration Plan

1. Add pure calculation/bounding tests and replace the old endpoints with the combined request-scoped `/api/stats` calculation.
2. Add page-local badge metadata while keeping the existing fill-up item schema.
3. Switch frontend consumers and refresh actions to the new contracts. Remove old per-vehicle statistics/history routes, handlers, client loaders, unused response types, and obsolete date-filter code; do not leave an unbounded fallback.
4. Run size/query-count, pagination, race, units, route-removal, and full pre-review checks. Update API documentation and release notes to identify the breaking route/response replacement and discontinued `from`/`to` filters.
5. Deploy the backend and embedded UI together. External API consumers must migrate to `/api/stats`; no compatibility aliases are provided. No data migration is required. Rolling back the binary restores the previous API without changing stored data.

These artifacts require human review before implementation. The discarded unbounded price-history fix is not restored or assumed.
