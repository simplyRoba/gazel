## Why

The stats engine (chunk 9) and dashboard enhancements (chunk 10) produce per-vehicle
efficiency, cost, and fuel-price data over time, but the UI only shows aggregate
summary cards and per-fill-up badges. Users have no way to spot trends, seasonal
patterns, or outliers. Adding time-series charts turns raw numbers into actionable
insight and is the natural next step for a fuel-tracking app.

## What Changes

- Integrate **LayerCake** (v10, Svelte-native, SVG) as the chart library -- validated
  via spike for Svelte 5 compat, CSS-token theming, and sparkline feasibility.
- Add three trend charts per vehicle: **efficiency over time** (line + area),
  **monthly cost** (bar), and **fuel price trend** (line).
- Add **sparkline background visuals** on two dashboard summary cards (cost per km
  and fuel price) using a standalone SVG sparkline component.
- Redesign dashboard layout for desktop/tablet: charts displayed **side-by-side**
  with the fill-up list, where only the fill-up list scrolls. On mobile: charts
  render as a **horizontal carousel** above the fill-up list.
- Build reusable chart layer components (Line, Area, Bar, AxisX, AxisY, Tooltip)
  and data-transformation utilities to map `SegmentHistory[]` to chart-ready shapes.
- Add Vitest tests for all data transformation logic.

## Capabilities

### New Capabilities
- `charts`: Chart components (LayerCake integration, reusable SVG layers, sparklines),
  data transformation utilities, and chart-specific layout patterns for the dashboard.

### Modified Capabilities
- `dashboard-stats`: Dashboard layout changes -- side-by-side charts + scrollable
  fill-up list on desktop/tablet, carousel + list on mobile. Sparkline backgrounds
  on cost-per-distance and fuel-price summary cards.

## Impact

- **New dependency**: `layercake` (runtime), `@types/d3-scale` (dev). LayerCake
  brings `d3-array`, `d3-color`, `d3-scale`, `d3-shape` as transitive deps.
  Install with `--legacy-peer-deps` due to stale TS ^5 peer dep (harmless with TS 6).
- **Frontend files**: new chart components under `ui/src/lib/components/charts/`,
  new data transform utils, dashboard page layout refactor.
- **Backend**: no changes -- existing `/api/vehicles/:id/stats/history` endpoint
  already returns all data needed for charting.
- **Responsive layout**: dashboard breakpoints at 768px (mobile/tablet) and 1280px
  (widescreen) remain; inner layout within the dashboard content area changes.
