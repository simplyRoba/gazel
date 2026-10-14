## Purpose

Dashboard statistics behavior, including data loading and refresh, fleet summary cards, responsive layout, and per-vehicle metrics.

## Requirements

### Requirement: Dashboard statistics lifecycle

The dashboard SHALL load fleet summaries, vehicle summaries, bounded charts, and bounded sparklines through one coordinated `/api/stats` request replacing the old per-vehicle summary/history requests. Fill-up listing SHALL remain independently paginated. Refreshes SHALL retain appropriate currently displayed values and expose loading and failure states without deriving summaries from sampled points.

#### Scenario: Statistics load succeeds
- **WHEN** dashboard data loads successfully
- **THEN** dashboard metrics, charts, and sparklines SHALL update together from that response
- **AND** any previous loading error SHALL be cleared
- **AND** the dashboard SHALL NOT use the removed per-vehicle summary/history endpoints or download complete segment or individual-price history

#### Scenario: Statistics load fails
- **WHEN** loading or refreshing dashboard data fails
- **THEN** previously displayed data for the same selection SHALL remain available
- **AND** a user-facing error SHALL be shown
- **AND** retained values SHALL keep the units and currency identified by their response

#### Scenario: Statistics retry begins
- **WHEN** a new load or refresh begins after a failure
- **THEN** the previous error SHALL be cleared
- **AND** the affected loading state SHALL be shown

#### Scenario: Fill-up data changes
- **WHEN** a fill-up is created, updated, or deleted
- **THEN** dashboard summaries, charts, sparklines, and affected page metadata SHALL refresh
- **AND** the list SHALL retain its existing paginated mutation/reload behavior

#### Scenario: Import, fleet, or settings change
- **WHEN** an import, vehicle mutation, or settings change affects displayed dashboard data
- **THEN** the displayed statistics SHALL be refreshed through one coordinated request
- **AND** changed units SHALL NOT relabel values computed under previous units

#### Scenario: Older response arrives last
- **WHEN** a newer refresh or selection request supersedes an outstanding request
- **THEN** the older result or failure SHALL NOT overwrite the newer data or error state
- **AND** an outstanding request SHALL NOT repopulate reset statistics state

#### Scenario: Exact summaries with sampled trends
- **WHEN** history exceeds chart or sparkline limits
- **THEN** summary values SHALL still match the full eligible history
- **AND** the client SHALL NOT calculate those values by averaging the sampled trend points

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

The charts panel SHALL display three charts for the selected vehicle from the bounded dashboard response. Chart cards SHALL remain visible when data is insufficient. Chart data and summary sparklines SHALL remain independent of the loaded fill-up list pages.

#### Scenario: Charts displayed
- **WHEN** a vehicle is selected via a chip or single-vehicle mode
- **THEN** the panel SHALL display monthly cost, monthly/yearly distance, and fuel price charts stacked vertically

#### Scenario: Vehicle selection changes chart data
- **WHEN** a different vehicle is selected
- **THEN** charts SHALL show that vehicle's bounded dashboard data
- **AND** data belonging to the previous vehicle SHALL NOT be displayed as the new vehicle's data

#### Scenario: Insufficient chart data
- **WHEN** the selected vehicle has insufficient data for a chart
- **THEN** both responsive layouts SHALL still display all three cards
- **AND** each chart SHALL retain its existing eligible-point threshold and empty-state message
- **AND** distance SHALL require two period points, cost SHALL accept one period, and fuel price SHALL accept the first eligible recorded fill as a visible dot

#### Scenario: Loaded fuel-price history
- **WHEN** bounded dashboard data for the selected vehicle loads or changes
- **THEN** both chart layouts SHALL render its supplied fuel-price series using the response units
- **AND** no complete price-history response or traversal of fill-up pages SHALL be needed
- **AND** cost, distance, and summary trends SHALL retain their existing calculation populations and thresholds

#### Scenario: Fill-up list continuation
- **WHEN** older list pages load on scroll
- **THEN** all chart points, values, axes, summary values, and sparkline paths SHALL remain unchanged
- **AND** no dashboard-data refresh or extra history request SHALL be triggered solely by pagination

#### Scenario: Chart history loading
- **WHEN** selected-vehicle statistics are loading and no matching response is currently available
- **THEN** compact and desktop chart layouts SHALL show skeleton placeholders

#### Scenario: No vehicles
- **WHEN** no vehicles exist
- **THEN** the dashboard SHALL show the existing vehicle empty state without chart cards
