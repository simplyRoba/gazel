## Purpose

Defines the dashboard's chart presentation, interaction, data-availability behavior, and calculation rules for efficiency, cost, distance, and fuel-price trends.

## Requirements

### Requirement: Consistent chart presentation

Dashboard charts SHALL present their data consistently, fit the available card width, and remain readable in every supported theme.

#### Scenario: Line-and-area presentation
- **WHEN** a trend is displayed as a line-and-area chart
- **THEN** the line SHALL connect data points in chronological order
- **AND** line joins and endpoints SHALL appear rounded
- **AND** a translucent area SHALL extend from the line to the horizontal axis

#### Scenario: Bar presentation
- **WHEN** a trend is displayed as a bar chart
- **THEN** each data point SHALL appear as one bar
- **AND** bars SHALL have square corners

#### Scenario: Time axis presentation
- **WHEN** a chart uses a continuous time axis
- **THEN** tick marks and date labels SHALL appear along the bottom
- **AND** date labels SHALL use abbreviated month and two-digit year formatting such as `Jan 25`
- **AND** subtle dashed vertical grid lines SHALL be displayed

#### Scenario: Category axis presentation
- **WHEN** a chart uses calendar-period categories such as months
- **THEN** tick marks and labels SHALL appear along the bottom without vertical grid lines

#### Scenario: Value axis presentation
- **WHEN** a chart displays a value axis
- **THEN** tick marks and value labels SHALL appear along the left side
- **AND** subtle dashed horizontal grid lines SHALL be displayed
- **AND** labels SHALL use the metric's required unit and locale formatting

#### Scenario: Theme changes
- **WHEN** the user changes between light and dark themes
- **THEN** chart lines, areas, bars, labels, indicators, and grid lines SHALL immediately use colors appropriate for the active theme

### Requirement: Chart titles and insufficient-data states

Each full-size chart SHALL display its title above the visualization and use the available content width. Charts that require a trend SHALL display an explanatory empty state until enough data exists.

#### Scenario: Trend has fewer than two points
- **WHEN** a line, area, or fuel-price trend has fewer than two eligible data points
- **THEN** the chart area SHALL display a centered message indicating that more fill-ups are needed

#### Scenario: Monthly cost has one period
- **WHEN** cost history falls within one calendar month
- **THEN** the monthly cost chart SHALL display one bar for that month

### Requirement: Chart point details

Interactive charts SHALL expose the nearest data point on hover or touch.

#### Scenario: Point selection
- **WHEN** the user hovers over or touches a chart area
- **THEN** the nearest data point's date and formatted metric value SHALL be displayed
- **AND** a vertical indicator line SHALL identify its horizontal position
- **AND** a dot SHALL highlight the selected point

#### Scenario: Detail positioning
- **WHEN** point details are displayed
- **THEN** their value and date text SHALL remain pinned to the chart's top-right corner
- **AND** the indicator line and highlight dot SHALL identify the selected point

#### Scenario: Point selection ends
- **WHEN** the user moves the pointer out of the chart area
- **THEN** the point details, indicator line, and highlight dot SHALL be hidden

### Requirement: Compact efficiency trend

The dashboard summary area SHALL display average efficiency with a low-contrast background trend when sufficient valid history exists.

#### Scenario: Efficiency summary data
- **WHEN** loaded vehicle history contains valid efficiency segments
- **THEN** the summary card SHALL display average efficiency formatted with the user's unit system
- **AND** segments where `is_valid` is `false` SHALL be excluded
- **AND** a chronological line-and-area trend SHALL appear behind the value when at least two valid segments exist

#### Scenario: Insufficient efficiency history
- **WHEN** fewer than two valid efficiency segments exist
- **THEN** no background trend line or area SHALL be displayed

### Requirement: Distance trend chart

The dashboard SHALL display a distance trend line-and-area chart for the selected vehicle.

#### Scenario: Monthly and yearly distance
- **WHEN** the selected vehicle has at least two history segments
- **THEN** the distance chart SHALL allow switching between monthly and yearly totals
- **AND** monthly mode SHALL display at most the latest 12 months
- **AND** the value axis and selected-point details SHALL format values with the user's distance unit
- **AND** the value-axis domain SHALL start at zero

### Requirement: Monthly cost trend chart

The dashboard SHALL display monthly cost totals as a bar chart for the selected vehicle.

#### Scenario: Monthly cost data
- **WHEN** the selected vehicle has segment history
- **THEN** each bar SHALL represent the total cost for one calendar month
- **AND** the category axis SHALL show month labels
- **AND** the value axis SHALL show values formatted with the user's currency

### Requirement: Fuel price trend chart

The dashboard SHALL display a fuel-price line chart for the selected vehicle.

#### Scenario: Fuel price data
- **WHEN** the selected vehicle has at least two eligible history segments
- **THEN** the chart title SHALL identify the currency and volume unit, such as `USD/L`
- **AND** value-axis labels SHALL show currency values without repeating the volume-unit suffix
- **AND** selected-point values SHALL include the volume-unit suffix
- **AND** the time axis SHALL show segment end dates

### Requirement: Chart calculation rules

Chart values SHALL be derived consistently from vehicle segment history and ordered chronologically.

#### Scenario: Efficiency trend calculation
- **WHEN** efficiency history is prepared for display
- **THEN** only segments where `is_valid` is `true` SHALL be included
- **AND** each point SHALL use the segment's end date and efficiency value

#### Scenario: Monthly cost calculation
- **WHEN** monthly cost history is prepared for display
- **THEN** segment costs SHALL be summed by calendar month
- **AND** month totals SHALL be ordered chronologically

#### Scenario: Monthly and yearly distance calculation
- **WHEN** monthly or yearly distance history is prepared for display
- **THEN** a segment's distance SHALL be distributed proportionally by calendar days across every month or year it spans
- **AND** period totals SHALL be ordered chronologically

#### Scenario: Fuel-price calculation
- **WHEN** fuel-price history is prepared for display
- **THEN** each eligible point SHALL use the segment end date and `cost / fuel` value
- **AND** segments with zero fuel SHALL be excluded

#### Scenario: Compact trend ordering
- **WHEN** a compact background trend is prepared for display
- **THEN** its points SHALL preserve the chronological order of eligible segment values
