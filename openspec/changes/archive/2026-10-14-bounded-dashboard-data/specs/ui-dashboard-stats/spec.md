# Spec Delta

## MODIFIED Requirements

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
