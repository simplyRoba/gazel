## Purpose

Dashboard statistics behavior, including data loading and refresh, fleet summary cards, responsive layout, and per-vehicle metrics.

## Requirements

### Requirement: Dashboard statistics lifecycle

The dashboard SHALL load the statistics and history needed for its current vehicle and fleet views, refresh affected data after fill-up changes, and expose loading or failure states without discarding previously displayed data.

#### Scenario: Statistics load succeeds

- **WHEN** statistics and history are loaded successfully
- **THEN** dashboard metrics and charts SHALL update with the returned data
- **AND** any previous loading error SHALL be cleared

#### Scenario: Statistics load fails

- **WHEN** loading or refreshing statistics fails
- **THEN** previously displayed statistics SHALL remain available
- **AND** a user-facing error SHALL be shown

#### Scenario: Statistics retry begins

- **WHEN** a new statistics load or refresh begins after a failure
- **THEN** the previous error SHALL be cleared
- **AND** the affected loading state SHALL be shown

#### Scenario: Fill-up data changes

- **WHEN** a fill-up is created, updated, or deleted
- **THEN** statistics and history for the affected vehicle SHALL refresh

### Requirement: Fleet summary cards

The dashboard SHALL display a row of summary cards showing aggregate metrics. For a single vehicle the cards show that vehicle's data; for multiple vehicles the cards aggregate across all vehicles. The average-efficiency and cost-per-distance summary cards SHALL display low-opacity background trends.

#### Scenario: Summary cards displayed

- **WHEN** the dashboard loads and at least one vehicle exists
- **THEN** summary cards SHALL display total fill-ups, total spent, average efficiency, and cost per distance unit
- **AND** monetary and efficiency values SHALL use the user's currency, unit, and locale settings

#### Scenario: Background trend on average-efficiency card

- **WHEN** the average-efficiency summary card has at least 2 valid history segments
- **THEN** a low-opacity trend SHALL appear behind the stat value
- **AND** the trend SHALL exclude invalid efficiency segments

#### Scenario: Background trend on cost-per-distance card

- **WHEN** the cost-per-distance summary card has at least 2 history segments
- **THEN** a low-opacity trend SHALL appear behind the stat value
- **AND** the trend SHALL use the selected vehicle's chronological `cost_per_distance` values

#### Scenario: Background trend with insufficient data

- **WHEN** a summary card with a background trend has fewer than 2 data points
- **THEN** the card SHALL show only the stat value

#### Scenario: No efficiency or distance data

- **WHEN** the aggregate has no valid efficiency or cost-per-distance value
- **THEN** the corresponding cards SHALL show a placeholder "—" instead of a number
- **AND** no corresponding background trend SHALL be displayed

#### Scenario: Loading state

- **WHEN** stats are being fetched
- **THEN** the summary cards SHALL display shimmer/skeleton placeholders

#### Scenario: Single vehicle layout

- **WHEN** only one vehicle exists
- **THEN** summary cards SHALL show that vehicle's data
- **AND** vehicle chips and per-vehicle stats row SHALL be hidden

#### Scenario: Multiple vehicles

- **WHEN** more than one vehicle exists
- **THEN** summary cards SHALL aggregate data across all vehicles
- **AND** vehicle chips and per-vehicle stats row SHALL be visible below the summary cards

### Requirement: Responsive summary card layout

The summary cards SHALL use a responsive grid layout that adapts to screen width, with compact sizing suitable for mobile.

#### Scenario: Viewport at least 640 pixels wide

- **WHEN** the viewport width is at least 640 pixels
- **THEN** the four summary cards SHALL display in four equal-width columns

#### Scenario: Viewport below 640 pixels wide

- **WHEN** the viewport width is below 640 pixels
- **THEN** the summary cards SHALL display in two equal-width columns

#### Scenario: Grid sizing

- **WHEN** the summary card grid is rendered
- **THEN** its equal-width columns SHALL expand to fill the available row width

### Requirement: Per-vehicle stats row

The dashboard SHALL display per-vehicle stats below the chip row when multiple vehicles exist. The stats shown SHALL match the same 4 metrics as the summary cards, scoped to the selected vehicle, without background trends.

#### Scenario: Stats shown for active vehicle

- **WHEN** a vehicle chip is selected and that vehicle has stats loaded
- **THEN** the dashboard SHALL display below the chip row: fill-ups count, total spent, average efficiency, and cost per distance unit — all for the selected vehicle only

#### Scenario: No data available

- **WHEN** the selected vehicle has no valid efficiency or cost-per-distance data
- **THEN** the average-efficiency and cost-per-distance displays SHALL show "—"

#### Scenario: Stats loading

- **WHEN** stats for the selected vehicle are still loading
- **THEN** the per-vehicle stats area SHALL show shimmer/skeleton placeholders

#### Scenario: Single vehicle

- **WHEN** only one vehicle exists
- **THEN** the per-vehicle stats row SHALL NOT be displayed (summary cards already show this vehicle's data)

### Requirement: Dashboard two-column layout

On viewports at least 960px wide, the dashboard content area SHALL use a two-column layout with charts on one side and the fill-up list on the other.

#### Scenario: Desktop layout

- **WHEN** the viewport width is at least 960px
- **THEN** the dashboard below the summary cards and chips/stats rows SHALL display in two columns
- **AND** the flexible-width left column SHALL contain the charts panel
- **AND** the right fill-up-list column SHALL be capped at 420px
- **AND** each column SHALL scroll independently when its content exceeds the available height
- **AND** scrolling the fill-up list SHALL NOT move the charts column

### Requirement: Dashboard compact layout

On viewports below 960px, charts SHALL render as a horizontal carousel above the fill-up list.

#### Scenario: Compact chart carousel

- **WHEN** the viewport width is below 960px
- **THEN** charts SHALL be displayed as horizontally swipeable cards
- **AND** each chart card SHALL be full-width within the carousel
- **AND** horizontal navigation SHALL settle with one card aligned in the viewport

#### Scenario: Carousel pagination indicator

- **WHEN** the chart carousel is displayed in the compact layout
- **THEN** dot indicators SHALL be shown below the carousel indicating the current chart
- **AND** the active dot SHALL be visually distinct (e.g., accent color)

#### Scenario: Fill-up list below carousel

- **WHEN** the compact layout is active
- **THEN** the fill-up list SHALL appear below the chart carousel
- **AND** the fill-up list SHALL scroll normally with the page

### Requirement: Charts panel content

The charts panel SHALL display three trend charts for the currently selected vehicle.

#### Scenario: Charts displayed

- **WHEN** a vehicle is selected (via chip or single-vehicle mode) and it has segment history
- **THEN** the charts panel SHALL display: monthly cost chart, monthly/yearly distance chart, and fuel price trend chart — stacked vertically

#### Scenario: Vehicle selection changes chart data

- **WHEN** the user selects a different vehicle via the chip row
- **THEN** the charts panel SHALL update to show charts for the newly selected vehicle

#### Scenario: Insufficient chart data

- **WHEN** the selected vehicle has fewer than 2 history segments
- **THEN** the chart carousel and chart column SHALL NOT be rendered
- **AND** the fill-up list SHALL use the available content width
