# Spec Delta

## MODIFIED Requirements

### Requirement: Combined statistics data

The API SHALL expose `GET /api/stats` with an optional positive `vehicle_id`. It SHALL return all-time fleet and per-vehicle summary values, the fleet efficiency sparkline, complete cost/distance period aggregates and recorded fuel prices for the selected vehicle, and its bounded cost-per-distance sparkline. It SHALL use the existing API authentication and JSON error contracts.

#### Scenario: Selected vehicle
- **WHEN** an existing vehicle is requested
- **THEN** one response SHALL contain the fleet summary, per-vehicle summaries, selected-vehicle summary, both complete cost/distance period modes, complete eligible fuel-price series, and both dashboard sparklines
- **AND** chart data for other vehicles SHALL NOT be included

#### Scenario: No selected vehicle
- **WHEN** `vehicle_id` is omitted
- **THEN** the response SHALL contain fleet and per-vehicle summaries and the fleet efficiency sparkline
- **AND** selected-vehicle data SHALL be `null`

#### Scenario: Invalid selection
- **WHEN** `vehicle_id` is malformed or not positive
- **THEN** the API SHALL return `400 Bad Request` with code `STATS_INVALID_VEHICLE_ID`
- **WHEN** a positive ID does not identify an existing vehicle
- **THEN** the API SHALL return `404 Not Found` with code `VEHICLE_NOT_FOUND`

#### Scenario: Empty database
- **WHEN** no vehicles exist and no vehicle is selected
- **THEN** totals SHALL be zero, undefined efficiency and distance ratios SHALL be `null`, and summary lists and chart series SHALL be empty

### Requirement: Exact dashboard summaries

All-time summary values SHALL use the complete eligible population rather than visible chart windows or sampled sparkline points. Existing tank-to-tank validity, unit conversion, rounding, fleet aggregation, and displayed average-efficiency semantics SHALL be preserved. Fuel prices SHALL remain independent of full-tank and odometer requirements.

#### Scenario: History outside chart windows
- **WHEN** older history is outside the currently visible monthly chart window
- **THEN** that history SHALL still contribute to applicable all-time totals and averages
- **AND** navigating chart windows SHALL NOT change those totals or averages

#### Scenario: Vehicles with different segment counts
- **WHEN** vehicles have different numbers of valid efficiency segments
- **THEN** the fleet displayed average efficiency SHALL give each valid segment equal weight rather than average vehicle averages

#### Scenario: First partial fill
- **WHEN** one partial fill-up has a positive finite fuel amount and a finite cost
- **THEN** it SHALL produce one fuel-price point without requiring a segment or a positive odometer
- **AND** its price SHALL use fuel volume normalized to the response's volume unit

#### Scenario: Invalid efficiency segment
- **WHEN** a segment contains a missed fill
- **THEN** it SHALL be excluded from efficiency averages and efficiency sparklines
- **AND** existing cost and distance eligibility rules SHALL remain unchanged

### Requirement: Single statistics API surface

The combined `/api/stats` endpoint SHALL provide summaries and complete chart history. The API SHALL NOT retain compatibility aliases for the old per-vehicle summary/history routes, introduce `/api/dashboard`, or require a separate individual-price-history route for dashboard charts.

#### Scenario: Statistics consumer migrates
- **WHEN** a consumer needs summaries or chart data
- **THEN** it SHALL obtain the combined response from `/api/stats`
- **AND** the old `/api/vehicles/{vehicle_id}/stats` and `/api/vehicles/{vehicle_id}/stats/history` paths SHALL no longer supply statistics responses or redirect to a compatibility alias

## REMOVED Requirements

### Requirement: Bounded dashboard series

**Reason**: Fixed chart limits make older periods and recorded prices inaccessible. Visible monthly windows belong to chart interaction, not to history truncation in the API.

**Migration**: Consume complete period `points` and fuel-price `points`/`trend` from `/api/stats`. Remove dependencies on period `source_count`, `source_range`, `displayed_range`, `omitted_older_periods`, and price `source_count`, `source_range`, `displayed_range`, `reduced`. Sparkline limits and metadata remain unchanged.

## ADDED Requirements

### Requirement: Complete dashboard chart history

Selected-vehicle chart data SHALL contain all monthly and yearly cost/distance aggregates and every eligible recorded fuel price in chronological order, without fixed history or point-count cutoffs. Period series SHALL contain `points`; fuel-price series SHALL contain `points` and `trend`, without omission or reduction metadata. Chart data SHALL NOT include raw fill-up lists or complete segment arrays.

#### Scenario: More than twelve periods
- **WHEN** eligible history spans 13 years
- **THEN** monthly and yearly cost/distance series SHALL include the earliest and latest applicable periods and every intervening calendar period
- **AND** missing periods within each series' source span SHALL have zero values
- **AND** no periods SHALL be added before that series' first applicable period or after its last applicable period

#### Scenario: Full distance intervals
- **WHEN** a segment crosses months or years
- **THEN** all overlapping periods SHALL receive their applicable calendar-day share of distance
- **AND** yearly values SHALL use direct yearly allocation and the existing integer-rounding rule, not sums of rounded monthly values

#### Scenario: More than 240 recorded prices
- **WHEN** a selected vehicle has 500 eligible recorded prices
- **THEN** its fuel-price `points` SHALL contain all 500 observations with their actual dates and unrounded normalized price values
- **AND** observations sharing dates or equal prices SHALL remain separate in date-then-ID order
- **AND** its trend SHALL use the complete eligible population

#### Scenario: Empty and short histories
- **WHEN** no eligible period or price data exists
- **THEN** the corresponding `points` SHALL be empty and the fuel-price `trend` SHALL be `null` when no fit exists
- **WHEN** only one eligible period or recorded price exists
- **THEN** that observation SHALL be returned without padding the history

### Requirement: Bounded dashboard sparklines

Each dashboard summary sparkline SHALL retain at most 60 representative eligible values and their original relative positions. Its reduction SHALL NOT affect exact summary values or remove points from full-size cost, distance, or fuel-price chart series.

#### Scenario: Long sparkline history
- **WHEN** more than 60 eligible values exist for a summary sparkline
- **THEN** at most 60 representative values SHALL be returned with the existing source-count and reduction metadata
- **AND** all eligible values SHALL still contribute to applicable exact summary calculations

#### Scenario: Short sparkline history
- **WHEN** at most 60 eligible values exist for a summary sparkline
- **THEN** all eligible values SHALL be returned in their original relative positions
