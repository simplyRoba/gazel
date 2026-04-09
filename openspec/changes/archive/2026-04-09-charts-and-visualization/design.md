## Context

The dashboard currently shows four fleet summary cards, an optional per-vehicle
stats row, vehicle chips, and a vertical fill-up card list. The backend already
serves `/api/vehicles/:id/stats/history` with per-segment efficiency, cost, and
fuel-price data. No chart library is installed. The user wants trend
visualizations integrated into the dashboard with a responsive side-by-side
layout on desktop/tablet and a carousel-then-list layout on mobile.

## Goals / Non-Goals

**Goals:**
- Integrate LayerCake as the charting library with reusable SVG chart layer
  components that inherit theme colors via CSS custom properties.
- Display three trend charts per vehicle: efficiency (line+area), monthly cost
  (bar), and fuel price (line).
- Add sparkline background visuals to the cost-per-distance and fuel-price
  summary cards.
- Refactor the dashboard layout so charts sit side-by-side with the fill-up list
  on desktop/tablet (only the list scrolls), and charts appear as a swipeable
  carousel above the list on mobile.
- Provide data transformation utilities with full Vitest coverage.

**Non-Goals:**
- Fleet-wide multi-vehicle overlay charts (single vehicle at a time).
- Date-range picker for chart filtering (backend supports it; UI deferred).
- Dedicated vehicle detail page for charts (charts live on the dashboard only).
- Interactive chart features beyond basic hover tooltip.
- Backend API changes.

## Decisions

### 1. Chart library: LayerCake v10

**Choice**: LayerCake over Chart.js or uPlot.

**Rationale**: Spike-validated. LayerCake renders to SVG, so chart elements can
use CSS custom properties (`var(--color-accent)`) and automatically switch
colors with `[data-theme]`. It is Svelte-native (`peerDependencies: svelte >=5`),
composable (no opinionated chart types), and tiny (~4kb). Chart.js and uPlot
render to Canvas, requiring JS-based color config and manual theme switching.

**Trade-off**: More boilerplate per chart (you build your own axes, lines, bars)
than Chart.js config objects. This is acceptable because gazel only needs three
chart types and the custom look aligns with the design system.

**Install note**: `npm install layercake --legacy-peer-deps` due to stale
`typescript ^5` peer dep declaration in LayerCake (harmless; passes svelte-check
and Vite build with TS 6).

### 2. Sparklines: standalone SVG, no LayerCake

**Choice**: Build sparklines as a pure Svelte component with raw SVG
`viewBox="0 0 100 100"` math. No LayerCake wrapper.

**Rationale**: Sparklines are embedded inside summary cards with absolute
positioning. They need no axes, no interaction, and must be extremely
lightweight. A 50-line standalone component avoids the overhead of LayerCake
context setup for what is essentially a static background path.

### 3. Dashboard layout: CSS Grid with sticky charts panel

**Choice**: On desktop/tablet (>768px), the dashboard content area becomes a
two-column CSS Grid. The left column holds the charts panel (position: sticky)
and the right column holds the scrollable fill-up list. On mobile (<=768px),
charts render as a horizontal carousel (`scroll-snap-type: x mandatory`) above
a normal fill-up list.

**Rationale**: The user explicitly requested side-by-side on desktop with only
entries scrolling, and carousel-then-list on mobile. CSS Grid + sticky
positioning achieves this without JavaScript scroll listeners. The carousel
uses native scroll-snap for smooth touch navigation, avoiding a carousel
library dependency.

**Alternatives considered**:
- Accordion/tabs for charts: rejected because the user wants charts always
  visible alongside the list.
- JavaScript-controlled layout: unnecessary; pure CSS handles the responsive
  switch.

### 4. Chart layer component structure

**Choice**: Place reusable chart layer components under
`ui/src/lib/components/charts/`:
- `Line.svelte` -- SVG `<path>` line
- `Area.svelte` -- SVG `<path>` filled area under a line
- `Bar.svelte` -- SVG `<rect>` bars
- `AxisX.svelte` -- time axis with formatted date ticks
- `AxisY.svelte` -- numeric axis with formatted value ticks
- `Tooltip.svelte` -- hover state overlay with data point info
- `Sparkline.svelte` -- standalone sparkline (no LayerCake)
- `ChartCard.svelte` -- wrapper providing chart container with title, sizing,
  and the LayerCake + Svg setup

**Rationale**: Each chart layer is a thin Svelte component that reads from
LayerCake context. This matches LayerCake's design philosophy and makes layers
reusable across the three chart types. `ChartCard` eliminates repeated
boilerplate (container div, LayerCake props, Svg layout).

### 5. Data transformation layer

**Choice**: Create `ui/src/lib/charts.ts` with pure functions:
- `toEfficiencyData(segments)` -- maps `SegmentHistory[]` to `{date, value}[]`
  for the efficiency line chart, filtering to valid segments only.
- `toMonthlyCostData(segments)` -- aggregates segment costs by month into
  `{month, value}[]` for the bar chart.
- `toFuelPriceData(segments)` -- maps segments to `{date, value}[]` where
  value is `cost / fuel` (cost per unit volume).
- `toSparklineData(segments, accessor)` -- generic mapper to `{x, y}[]` for
  sparkline backgrounds.

**Rationale**: Keeping data transformation as pure functions separate from
components makes them independently testable with Vitest and reusable if charts
appear on other pages in the future.

### 6. LayerCake context typing

**Choice**: Use `as` cast on `getContext('LayerCake')` in each layer component
with a shared type definition.

**Rationale**: LayerCake's `getContext` returns `unknown` in strict TypeScript.
The spike confirmed that casting to the expected shape (Readable stores for
data, scales, dimensions) works cleanly. A shared
`LayerCakeContext` type alias in `charts.ts` avoids repeating the cast shape.

## Risks / Trade-offs

- **LayerCake TS ^5 peer dep** → Mitigated by `--legacy-peer-deps` install.
  Spike confirmed 0 build errors with TS 6. Monitor for LayerCake releases that
  update the peer dep declaration.
- **d3 bundle size increase** → LayerCake brings d3-array, d3-color, d3-scale,
  d3-shape (~25kb gzipped total). Acceptable for a charting feature; these are
  tree-shakable.
- **Carousel without a library** → Native `scroll-snap` is well-supported but
  lacks pagination dots natively. Implement minimal dots indicator with CSS/HTML
  (no library).
- **No date-range filtering UI** → Users cannot scope charts to a time range.
  The backend supports `from`/`to` query params, so adding a filter later is
  straightforward. This avoids scope creep now.
- **Charts render empty with <2 data points** → Show an empty-state message
  inside the chart card when insufficient data exists for a meaningful chart.
