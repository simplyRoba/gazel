## Context

The dashboard (`/`) currently shows vehicle chips, a fill-up card list, and a global CTA button. The backend stats engine (chunk 9) is complete — `GET /api/vehicles/:id/stats` returns aggregate metrics and `GET /api/vehicles/:id/stats/history` returns per-segment data points. However, the frontend has zero stats integration: no TypeScript types, no API client functions, no stats store, and no visual stats display.

The existing UI follows established patterns: Svelte 5 runes for stores, centralized `api.ts` for backend calls, `formatEfficiency`/`formatCurrency`/`formatDistance` utilities, and CSS Modules for component scoping.

## Goals / Non-Goals

**Goals:**

- Surface summary metrics (total distance, fill-ups, cost per km, fuel price) at the top of the dashboard — same 4 stats for single or multiple vehicles
- For multiple vehicles: show vehicle chips and per-vehicle stats row with the same 4 metrics scoped to the selected vehicle
- Display efficiency badges on individual fill-up cards by mapping segment history to fill-ups
- Build a reactive stats store that caches per-vehicle stats and invalidates on fill-up mutations
- Keep the dashboard responsive — summary cards reflow to a single column on mobile

**Non-Goals:**

- Charts or trend visualization (chunk 11)
- New backend endpoints or API changes — all data already available
- Vehicle detail page — stats are shown on the dashboard only
- Date-range filtering in the UI — use all-time stats initially

## Decisions

### Stats store architecture

**Decision:** Create a single `stats.svelte.ts` store that manages both `VehicleStats` (summary) and `SegmentHistory[]` (per-segment) per vehicle, using `SvelteMap<number, ...>` caches keyed by vehicle ID — matching the fill-up store pattern.

**Why:** Consistent with the existing `fillups.svelte.ts` pattern. A single store reduces import churn and both data shapes share the same lifecycle (load when vehicle is selected, invalidate on fill-up mutation).

**Alternative considered:** Fetch stats inline in components without a store. Rejected because multiple components (summary cards, chips, fill-up cards) need the same data, and a store provides caching and deduplication.

### Fleet summary data loading

**Decision:** On dashboard mount, fetch stats for all vehicles in parallel (`Promise.all`). The fleet summary (total distance, fill-ups, cost/distance, fuel price) is computed client-side from individual vehicle stats. For a single vehicle, the same summary cards display that vehicle's data without chips or a per-vehicle row.

**Why:** There is no fleet-wide stats endpoint, and adding one would require backend changes (out of scope). With a small vehicle count (personal use), parallel per-vehicle requests are acceptable. Client-side aggregation keeps the implementation simple. Hiding chips and the per-vehicle row for single-vehicle users avoids redundant display.

**Alternative considered:** A dedicated `GET /api/stats/fleet` endpoint. Rejected to avoid backend changes in this chunk and because the vehicle count in a personal fuel tracker is inherently small.

### Efficiency badge on fill-up cards

**Decision:** Map segment history entries to individual fill-ups by matching `end_date`/`end_odometer`. Each full-tank fill-up that terminates a segment gets the segment's efficiency displayed as a badge.

**Why:** The segment history returns `end_date` and `end_odometer` which correspond to a specific full-tank fill-up. This is a straightforward lookup. Only full-tank fills that end a valid segment show the badge, which is the natural semantics of tank-to-tank efficiency.

**Alternative considered:** Show efficiency on every fill-up card by interpolating. Rejected because partial-tank fills don't have meaningful standalone efficiency; the tank-to-tank method is the correct model.

### No extracted fill-up card component

**Decision:** Keep fill-up card markup inline in `+page.svelte` and add the efficiency badge there. Do not extract a `FillupCard.svelte` component in this change.

**Why:** Extraction is a refactor orthogonal to the feature goal. The current inline approach works fine and avoids scope creep. If the card grows further (charts, actions), extraction can happen in a later change.

### Responsive summary card layout

**Decision:** Use CSS Grid with `auto-fill` / `minmax(140px, 1fr)` for summary cards. Cards use compact styling (`font-md` for values, `font-xs` for labels, `space-3` padding) and reflow naturally on narrow screens.

**Why:** Pure CSS solution with no JS media queries. Compact sizing keeps cards useful on mobile without wasting space. Works with the existing CSS approach and the 4px spacing grid from the design system.

## Risks / Trade-offs

- **[N+1 stats requests]** Fetching stats per-vehicle on mount could be slow with many vehicles. → Mitigation: Personal tracker, vehicle count is small. If it becomes a problem, a fleet endpoint can be added later.
- **[Stale stats after fill-up mutation]** Stats become stale when a fill-up is created/edited/deleted. → Mitigation: Invalidate and refetch stats for the affected vehicle after any fill-up mutation succeeds.
- **[Segment-to-fillup mapping reliability]** Matching segments to fill-ups via `end_date`/`end_odometer` could fail if two fill-ups share the same date and odometer. → Mitigation: Extremely unlikely for real data. The combination of date + odometer is effectively unique.
- **[No loading state for summary cards]** Stats load asynchronously and may appear after the chip row. → Mitigation: Show shimmer placeholders for summary cards while stats are loading, consistent with existing loading patterns.
