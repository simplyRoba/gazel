## Why

The dashboard currently displays vehicle chips and a fill-up card list but shows zero aggregate data. With the stats calculation engine (chunk 9) now complete, users have no way to see efficiency, cost, or summary metrics at a glance. Surfacing these numbers on the dashboard turns it from a log-browsing view into an at-a-glance fuel intelligence screen.

## What Changes

- Add summary cards showing: total distance, fill-up count, cost per distance unit, and fuel price — same 4 metrics whether the user has one vehicle or many
- For single-vehicle users: show summary cards and fill-up list only (no chips, no redundant per-vehicle row)
- For multi-vehicle users: show aggregated summary cards, vehicle chips, per-vehicle stats row (same 4 metrics scoped to selected vehicle), and fill-up list
- Add an efficiency badge on individual fill-up cards (e.g., "12.5 km/L") by consuming segment history from `GET /api/vehicles/:id/stats/history`
- Introduce a stats store to fetch/cache vehicle stats and segment history on the frontend
- Add stats-related TypeScript types and API client functions (`VehicleStats`, `SegmentHistory`, `fetchVehicleStats`, `fetchVehicleStatsHistory`)
- Make summary cards responsive — grid reflows to single column on mobile
- Add Vitest tests for the stats store, stats display components, and formatting integration

## Capabilities

### New Capabilities

- `dashboard-stats`: Summary cards (total distance, fill-ups, cost/distance, fuel price), adaptive layout for single vs multi-vehicle, per-vehicle stats row, stats store, stats API client types/functions, and responsive summary layout on the dashboard

### Modified Capabilities

- `fillup-ui`: Fill-up cards gain an efficiency badge per segment, requiring stats history data and the `formatEfficiency` formatter

## Impact

- **Frontend new files:** `ui/src/lib/stores/stats.svelte.ts` (stats store), `ui/src/lib/stats.ts` (fleet summary computation, segment-to-fillup mapping), stats-related Vitest tests
- **Frontend modified files:** `ui/src/lib/api.ts` (add `VehicleStats`, `SegmentHistory` types + fetch functions), `ui/src/routes/+page.svelte` (summary cards, per-vehicle stats, efficiency badge on fill-up cards, single vs multi-vehicle layout), dashboard CSS (responsive grid for summary cards)
- **Backend:** No backend changes — all data is already exposed by the vehicle stats endpoints
- **Dependencies:** No new dependencies — uses existing formatting utilities and API patterns
