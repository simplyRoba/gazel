## ADDED Requirements

### Requirement: Chart library integration

The application SHALL use LayerCake v10 as the chart library, installed as a runtime
dependency alongside `@types/d3-scale` as a dev dependency.

#### Scenario: LayerCake installed

- **WHEN** the UI dependencies are installed
- **THEN** `layercake` SHALL be present in `dependencies` in `ui/package.json`
- **AND** `@types/d3-scale` SHALL be present in `devDependencies`

#### Scenario: Installation compatibility

- **WHEN** `npm install` is run
- **THEN** the install SHALL use `--legacy-peer-deps` to resolve the stale TypeScript ^5 peer dep in LayerCake
- **AND** `svelte-check` and `vite build` SHALL pass with zero errors

### Requirement: Chart layer components

The application SHALL provide reusable SVG chart layer components under
`ui/src/lib/components/charts/` that render inside LayerCake's `Svg` layout.

#### Scenario: Line component

- **WHEN** a `Line` component is placed inside a LayerCake `Svg` layout
- **THEN** it SHALL render an SVG `<path>` connecting all data points in order
- **AND** the stroke color SHALL default to `var(--color-accent)` (CSS custom property)
- **AND** stroke-linejoin and stroke-linecap SHALL be `round`

#### Scenario: Area component

- **WHEN** an `Area` component is placed inside a LayerCake `Svg` layout
- **THEN** it SHALL render a filled SVG `<path>` from the data line down to the x-axis
- **AND** the fill color SHALL default to `var(--color-accent-subtle)`
- **AND** the fill opacity SHALL default to `0.3`

#### Scenario: Bar component

- **WHEN** a `Bar` component is placed inside a LayerCake `Svg` layout with a band scale
- **THEN** it SHALL render SVG `<rect>` elements for each data point
- **AND** the fill color SHALL default to `var(--color-accent)`
- **AND** bars SHALL have rounded top corners (border-radius via `rx` attribute)

#### Scenario: AxisX component

- **WHEN** an `AxisX` component is placed inside a LayerCake `Svg` layout
- **THEN** it SHALL render tick marks and labels along the bottom of the chart
- **AND** tick labels for time scales SHALL format dates as abbreviated month + 2-digit year (e.g., "Jan 25")
- **AND** grid lines SHALL use `var(--color-border-subtle)` with dashed stroke

#### Scenario: AxisY component

- **WHEN** an `AxisY` component is placed inside a LayerCake `Svg` layout
- **THEN** it SHALL render tick marks and labels along the left side of the chart
- **AND** the component SHALL accept a `format` prop for custom tick label formatting
- **AND** grid lines SHALL use `var(--color-border-subtle)` with dashed stroke

#### Scenario: Theme integration

- **WHEN** the user switches between light and dark themes
- **THEN** all chart colors using CSS custom properties SHALL update automatically
- **AND** no JavaScript theme detection SHALL be needed in chart components

### Requirement: ChartCard wrapper component

The application SHALL provide a `ChartCard` component that wraps LayerCake setup
boilerplate into a reusable container.

#### Scenario: ChartCard rendering

- **WHEN** a `ChartCard` is rendered with data, x/y accessors, and layer components
- **THEN** it SHALL display a title label above the chart
- **AND** the chart container SHALL expand to fill the card width
- **AND** the chart height SHALL be a configurable prop with a sensible default

#### Scenario: ChartCard with insufficient data

- **WHEN** a `ChartCard` receives fewer than 2 data points
- **THEN** it SHALL display a centered empty-state message instead of the chart
- **AND** the message SHALL indicate that more fill-ups are needed

### Requirement: Tooltip component

The application SHALL provide a `Tooltip` component for displaying data point
details on hover.

#### Scenario: Hover interaction

- **WHEN** the user hovers over or touches a chart area
- **THEN** the tooltip SHALL display the nearest data point's values (date, metric value)
- **AND** a vertical indicator line SHALL appear at the hovered x position
- **AND** a dot SHALL highlight the data point on the line

#### Scenario: Tooltip positioning

- **WHEN** the tooltip is displayed
- **THEN** it SHALL position itself near the hovered point without overflowing the chart bounds

#### Scenario: Tooltip dismiss

- **WHEN** the user moves the pointer out of the chart area
- **THEN** the tooltip, indicator line, and highlight dot SHALL be hidden

### Requirement: Sparkline component

The application SHALL provide a standalone `Sparkline` component that renders a
minimal area+line chart as a background visual, without requiring LayerCake.

#### Scenario: Sparkline rendering

- **WHEN** a `Sparkline` is rendered with an array of `{x, y}` data points
- **THEN** it SHALL render an SVG with `viewBox="0 0 100 100"` and `preserveAspectRatio="none"`
- **AND** it SHALL draw a filled area path and a line path
- **AND** the stroke color SHALL default to `var(--color-accent)`
- **AND** the fill color SHALL default to `var(--color-accent)` with low opacity

#### Scenario: Sparkline with fewer than 2 points

- **WHEN** a `Sparkline` receives fewer than 2 data points
- **THEN** it SHALL render an empty SVG (no paths)

### Requirement: Efficiency trend chart

The dashboard SHALL display an efficiency trend line chart for the selected vehicle.

#### Scenario: Efficiency chart data

- **WHEN** the selected vehicle has valid segment history with at least 2 valid segments
- **THEN** the efficiency chart SHALL display a line+area chart
- **AND** the x-axis SHALL show segment end dates
- **AND** the y-axis SHALL show efficiency values formatted with the user's unit system (e.g., "km/L" or "mpg")
- **AND** the y-axis domain SHALL start at 0

#### Scenario: Invalid segments excluded

- **WHEN** segment history contains segments where `is_valid` is `false`
- **THEN** those segments SHALL be excluded from the efficiency chart

### Requirement: Monthly cost trend chart

The dashboard SHALL display a monthly cost bar chart for the selected vehicle.

#### Scenario: Monthly cost chart data

- **WHEN** the selected vehicle has segment history spanning multiple months
- **THEN** the monthly cost chart SHALL display a bar chart
- **AND** each bar SHALL represent the total cost for one calendar month
- **AND** the x-axis SHALL show month labels
- **AND** the y-axis SHALL show cost values formatted with the user's currency

#### Scenario: Single month data

- **WHEN** all segment history falls within a single month
- **THEN** the chart SHALL display a single bar for that month

### Requirement: Fuel price trend chart

The dashboard SHALL display a fuel price trend line chart for the selected vehicle.

#### Scenario: Fuel price chart data

- **WHEN** the selected vehicle has segment history with at least 2 segments
- **THEN** the fuel price chart SHALL display a line chart
- **AND** the y-axis SHALL show cost per volume unit (e.g., "$/L" or "$/gal")
- **AND** the x-axis SHALL show segment end dates

### Requirement: Data transformation utilities

The application SHALL provide pure data transformation functions in
`ui/src/lib/charts.ts` for converting `SegmentHistory[]` into chart-ready data shapes.

#### Scenario: Efficiency data transformation

- **WHEN** `toEfficiencyData(segments)` is called with a `SegmentHistory[]`
- **THEN** it SHALL return an array of `{date: Date, value: number}` objects
- **AND** only segments where `is_valid` is `true` SHALL be included
- **AND** `date` SHALL be parsed from `end_date` and `value` SHALL be `efficiency`

#### Scenario: Monthly cost data transformation

- **WHEN** `toMonthlyCostData(segments)` is called with a `SegmentHistory[]`
- **THEN** it SHALL return an array of `{month: string, value: number}` objects
- **AND** costs SHALL be summed per calendar month (YYYY-MM format)
- **AND** the array SHALL be sorted chronologically

#### Scenario: Fuel price data transformation

- **WHEN** `toFuelPriceData(segments)` is called with a `SegmentHistory[]`
- **THEN** it SHALL return an array of `{date: Date, value: number}` objects
- **AND** `value` SHALL be `cost / fuel` for each segment
- **AND** segments with zero fuel SHALL be excluded

#### Scenario: Sparkline data transformation

- **WHEN** `toSparklineData(segments, accessor)` is called
- **THEN** it SHALL return an array of `{x: number, y: number}` objects
- **AND** `x` SHALL be the segment index and `y` SHALL be the value from the accessor function

### Requirement: Data transformation test coverage

All data transformation functions SHALL have Vitest test coverage.

#### Scenario: Edge case tests

- **WHEN** tests are run for data transformation functions
- **THEN** they SHALL cover: empty input, single segment, segments with `is_valid: false`, segments with zero fuel, multiple months of data, and correct date parsing

### Requirement: LayerCake context typing

Chart layer components SHALL use typed access to the LayerCake context to satisfy
TypeScript strict mode.

#### Scenario: Context type cast

- **WHEN** a chart layer component calls `getContext('LayerCake')`
- **THEN** it SHALL cast the result using a shared `LayerCakeContext` type
- **AND** `svelte-check` SHALL pass with zero type errors in chart components
