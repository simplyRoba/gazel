## 1. Dependencies and setup

- [x] 1.1 Add `layercake` to `dependencies` and `@types/d3-scale` to `devDependencies` in `ui/package.json` (install with `--legacy-peer-deps`)
- [x] 1.2 Define shared `LayerCakeContext` type alias in `ui/src/lib/charts.ts` for typed `getContext('LayerCake')` casts

## 2. Data transformation utilities

- [x] 2.1 Implement `toEfficiencyData(segments)` in `ui/src/lib/charts.ts` — maps `SegmentHistory[]` to `{date, value}[]`, filtering to `is_valid === true`
- [x] 2.2 Implement `toMonthlyCostData(segments)` — aggregates segment costs by calendar month into `{month, value}[]`, sorted chronologically
- [x] 2.3 Implement `toFuelPriceData(segments)` — maps segments to `{date, value}[]` where value is `cost / fuel`, excluding zero-fuel segments
- [x] 2.4 Implement `toSparklineData(segments, accessor)` — generic mapper to `{x, y}[]` for sparkline backgrounds

## 3. Data transformation tests

- [x] 3.1 Write Vitest tests for `toEfficiencyData`: empty input, single segment, mixed valid/invalid segments, correct date parsing
- [x] 3.2 Write Vitest tests for `toMonthlyCostData`: empty input, single month, multiple months, chronological ordering
- [x] 3.3 Write Vitest tests for `toFuelPriceData`: empty input, zero-fuel exclusion, correct cost/fuel calculation
- [x] 3.4 Write Vitest tests for `toSparklineData`: empty input, correct index-based x values, custom accessor

## 4. Chart layer components

- [x] 4.1 Create `Line.svelte` under `ui/src/lib/components/charts/` — SVG path line with CSS custom property stroke color, typed LayerCake context
- [x] 4.2 Create `Area.svelte` — SVG filled area path under line, accent-subtle fill with 0.3 opacity
- [x] 4.3 Create `Bar.svelte` — SVG rect bars with band scale support, rounded top corners, accent fill
- [x] 4.4 Create `AxisX.svelte` — time axis ticks formatted as "Mon YY", dashed grid lines using `--color-border-subtle`
- [x] 4.5 Create `AxisY.svelte` — numeric axis with configurable `format` prop, dashed grid lines
- [x] 4.6 Create `Tooltip.svelte` — hover/touch overlay showing nearest data point values, vertical indicator line, highlight dot
- [x] 4.7 Create `Sparkline.svelte` — standalone SVG sparkline (no LayerCake), `viewBox="0 0 100 100"`, accent colors

## 5. ChartCard wrapper

- [x] 5.1 Create `ChartCard.svelte` — wrapper with title label, configurable height, LayerCake + Svg boilerplate, empty-state when <2 data points

## 6. Dashboard chart panels

- [x] 6.1 Create `EfficiencyChart.svelte` — composed ChartCard with Line + Area + AxisX + AxisY + Tooltip, fed by `toEfficiencyData`, y-axis formatted with user's efficiency unit
- [x] 6.2 Create `MonthlyCostChart.svelte` — composed ChartCard with Bar + AxisX + AxisY + Tooltip, fed by `toMonthlyCostData`, y-axis formatted with user's currency
- [x] 6.3 Create `FuelPriceChart.svelte` — composed ChartCard with Line + AxisX + AxisY + Tooltip, fed by `toFuelPriceData`, y-axis formatted with user's currency/volume
- [x] 6.4 Create `ChartsPanel.svelte` — stacks all three chart components vertically, accepts vehicle's segment history and unit metadata

## 7. Dashboard layout refactor

- [x] 7.1 Refactor `+page.svelte` desktop/tablet layout (>768px): two-column CSS Grid below summary cards — left column sticky charts panel (~40%), right column scrollable fill-up list (~60%)
- [x] 7.2 Refactor `+page.svelte` mobile layout (<=768px): horizontal chart carousel with `scroll-snap-type: x mandatory`, each chart full-width, carousel above fill-up list
- [x] 7.3 Add dot pagination indicators below mobile carousel — active dot styled with accent color
- [x] 7.4 Wire `ChartsPanel` to selected vehicle's segment history from stats store, update on vehicle chip selection

## 8. Sparkline integration on summary cards

- [x] 8.1 Add sparkline background to cost-per-distance summary card — positioned absolutely, low opacity, data from `toSparklineData(segments, d => d.cost_per_distance)`
- [x] 8.2 Add sparkline background to fuel-price summary card — same pattern, data from `toSparklineData(segments, d => d.cost / d.fuel)`
- [x] 8.3 Conditionally hide sparklines when fewer than 2 data points available

## 9. Quality and verification

- [x] 9.1 Run `npm run check --prefix ui` — zero errors
- [x] 9.2 Run `npm run lint --prefix ui` — zero errors
- [x] 9.3 Run `npm run format:check --prefix ui` — pass
- [x] 9.4 Run `npm run test --prefix ui` — all tests pass including new chart transform tests
- [x] 9.5 Run `cargo fmt -- --check` — pass
- [x] 9.6 Run `cargo clippy -- -D warnings` — pass
- [x] 9.7 Run `cargo test` — all tests pass
