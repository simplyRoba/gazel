## MODIFIED Requirements

### Requirement: Fleet summary cards

The dashboard SHALL display a row of summary cards showing aggregate metrics. For a single vehicle the cards show that vehicle's data; for multiple vehicles the cards aggregate across all vehicles. The cost-per-distance and fuel-price summary cards SHALL display sparkline background visuals.

#### Scenario: Summary cards displayed

- **WHEN** the dashboard loads and at least one vehicle exists
- **THEN** summary cards SHALL be displayed showing: total distance (formatted with `formatDistance`), total fill-ups count, cost per distance unit (formatted with `formatCurrency` and the user's distance_unit), and fuel price (total cost divided by total fuel, formatted with `formatCurrency` and the user's volume_unit)

#### Scenario: Sparkline on cost-per-distance card

- **WHEN** the cost-per-distance summary card is rendered and the selected vehicle has segment history
- **THEN** a `Sparkline` component SHALL be rendered as a background visual behind the stat value
- **AND** the sparkline data SHALL be derived from the vehicle's segment history cost_per_distance values
- **AND** the sparkline SHALL have low opacity so it does not compete with the stat text

#### Scenario: Sparkline on fuel-price card

- **WHEN** the fuel-price summary card is rendered and the selected vehicle has segment history
- **THEN** a `Sparkline` component SHALL be rendered as a background visual behind the stat value
- **AND** the sparkline data SHALL be derived from the vehicle's segment history fuel price values (cost / fuel)

#### Scenario: Sparkline with insufficient data

- **WHEN** a summary card with sparkline has fewer than 2 data points
- **THEN** the sparkline SHALL NOT be rendered (card shows stat value only)

#### Scenario: No distance or fuel data

- **WHEN** all vehicles have zero fill-ups (total_distance and total_fuel are 0)
- **THEN** the cost per distance and fuel price cards SHALL show a placeholder "—" instead of a number
- **AND** no sparkline SHALL be rendered

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

#### Scenario: Wide viewport

- **WHEN** the viewport is wide enough (desktop)
- **THEN** summary cards SHALL display in a single row

#### Scenario: Narrow viewport

- **WHEN** the viewport is narrow (mobile)
- **THEN** summary cards SHALL reflow into fewer columns (2x2 or single column)

#### Scenario: Grid sizing

- **WHEN** the summary card grid is rendered
- **THEN** each card SHALL have a minimum width of approximately 140px and expand to fill available space

## ADDED Requirements

### Requirement: Dashboard two-column layout

On desktop and tablet viewports (>768px), the dashboard content area SHALL use a
two-column layout with charts on one side and the fill-up list on the other.

#### Scenario: Desktop/tablet layout

- **WHEN** the viewport width is greater than 768px
- **THEN** the dashboard below the summary cards and chips/stats rows SHALL display as a two-column CSS Grid
- **AND** the left column SHALL contain the charts panel
- **AND** the right column SHALL contain the fill-up list
- **AND** the charts panel SHALL be sticky (position: sticky) so it remains visible while scrolling
- **AND** only the fill-up list column SHALL scroll

#### Scenario: Column proportions

- **WHEN** the two-column layout is active
- **THEN** the charts panel SHALL occupy approximately 40% of the width
- **AND** the fill-up list SHALL occupy approximately 60% of the width

### Requirement: Dashboard mobile layout

On mobile viewports (<=768px), charts SHALL render as a horizontal carousel above
the fill-up list.

#### Scenario: Mobile chart carousel

- **WHEN** the viewport width is 768px or less
- **THEN** charts SHALL be displayed as horizontally swipeable cards using CSS scroll-snap
- **AND** each chart card SHALL be full-width within the carousel
- **AND** scroll-snap-type SHALL be set to `x mandatory` for crisp snapping

#### Scenario: Carousel pagination indicator

- **WHEN** the chart carousel is displayed on mobile
- **THEN** dot indicators SHALL be shown below the carousel indicating the current chart
- **AND** the active dot SHALL be visually distinct (e.g., accent color)

#### Scenario: Fill-up list below carousel

- **WHEN** the mobile layout is active
- **THEN** the fill-up list SHALL appear below the chart carousel
- **AND** the fill-up list SHALL scroll normally with the page

### Requirement: Charts panel content

The charts panel SHALL display three trend charts for the currently selected vehicle.

#### Scenario: Charts displayed

- **WHEN** a vehicle is selected (via chip or single-vehicle mode) and it has segment history
- **THEN** the charts panel SHALL display: efficiency trend chart, monthly cost chart, and fuel price trend chart — stacked vertically

#### Scenario: Vehicle selection changes chart data

- **WHEN** the user selects a different vehicle via the chip row
- **THEN** the charts panel SHALL update to show charts for the newly selected vehicle

#### Scenario: No chart data

- **WHEN** the selected vehicle has no segment history or fewer than 2 data points
- **THEN** the charts panel SHALL display an empty state indicating more fill-ups are needed
