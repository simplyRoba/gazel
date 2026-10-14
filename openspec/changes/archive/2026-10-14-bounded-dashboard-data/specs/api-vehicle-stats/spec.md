# Spec Delta

## ADDED Requirements

### Requirement: Combined statistics data

The API SHALL expose `GET /api/stats` with an optional positive `vehicle_id`. It SHALL return all-time fleet and per-vehicle summary values, the fleet efficiency sparkline, and bounded chart and cost-per-distance sparkline data for the selected vehicle. It SHALL use the existing API authentication and JSON error contracts.

#### Scenario: Selected vehicle
- **WHEN** an existing vehicle is requested
- **THEN** one response SHALL contain the fleet summary, per-vehicle summaries, selected-vehicle summary, both cost/distance period modes, fuel-price series, and both dashboard sparklines
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

### Requirement: Bounded dashboard series

Dashboard chart-series sizes SHALL remain bounded independently of fill-up count. Each cost/distance mode SHALL contain at most 12 periods, the fuel-price series at most 240 points, and each sparkline at most 60 points. Series SHALL identify their displayed range and whether source history was reduced. Dashboard responses SHALL NOT contain raw fill-up lists or complete segment arrays.

#### Scenario: Growing history
- **WHEN** a vehicle has 100,000 eligible fill-ups rather than 100
- **THEN** its response SHALL obey the same series limits
- **AND** additional history SHALL NOT add one response record per fill-up or segment

#### Scenario: Period bounds
- **WHEN** cost or distance history spans more than 12 months or years
- **THEN** that mode SHALL contain only the latest 12 calendar periods ending at its latest data period
- **AND** missing periods inside that window SHALL be zero-filled
- **AND** the response SHALL indicate that earlier periods are not displayed

#### Scenario: Fuel-price reduction
- **WHEN** more than 240 eligible recorded prices exist
- **THEN** the returned series SHALL preserve the first and last eligible prices and the history's minimum and maximum prices
- **AND** retained points SHALL preserve chronological order and their actual recorded price values
- **AND** all-time source count and reduction status SHALL be provided

#### Scenario: Small series
- **WHEN** an eligible series fits within its point limit
- **THEN** all of its points SHALL be returned without reduction

### Requirement: Exact dashboard summaries

All-time summary values SHALL use the complete eligible population rather than displayed windows or sampled chart points. Existing tank-to-tank validity, unit conversion, rounding, fleet aggregation, and displayed average-efficiency semantics SHALL be preserved. Fuel prices SHALL remain independent of full-tank and odometer requirements.

#### Scenario: History outside chart windows
- **WHEN** older history is omitted from a displayed chart window
- **THEN** that history SHALL still contribute to applicable all-time totals and averages

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

### Requirement: Fresh statistics responses

Each statistics request SHALL return results from a coherent current database view and identify its distance unit, volume unit, and currency. Responses SHALL use `Cache-Control: no-store`. A request begun after a committed data or settings change SHALL reflect that change without an expiry delay or a separate invalidation operation.

#### Scenario: Mutation followed by statistics request
- **WHEN** a statistics request begins after a fill-up create, edit, or delete has committed
- **THEN** its summaries and charts SHALL reflect that change

#### Scenario: Import or vehicle changes
- **WHEN** a statistics request begins after an import or vehicle mutation commits
- **THEN** its data SHALL reflect the current fleet and affected vehicles
- **AND** reused vehicle IDs SHALL NOT expose statistics from replaced database contents

#### Scenario: Settings change
- **WHEN** a statistics request begins after distance, volume, or currency settings change
- **THEN** its values and labels SHALL coherently use the response's current units

#### Scenario: External write
- **WHEN** a statistics request begins after database changes commit outside the application
- **THEN** its results SHALL reflect those committed changes without waiting for a cache timeout

#### Scenario: Calculation failure
- **WHEN** statistics calculation fails
- **THEN** the endpoint SHALL return the existing safe infrastructure-error contract
- **AND** it SHALL NOT substitute an older computed response

### Requirement: Single statistics API surface

The combined `/api/stats` endpoint SHALL replace the old per-vehicle summary and segment-history routes. The API SHALL NOT retain compatibility aliases, introduce `/api/dashboard`, or expose another unbounded individual-price-history route for this data.

#### Scenario: Statistics consumer migrates
- **WHEN** a consumer needs summaries or chart data
- **THEN** it SHALL obtain the combined bounded response from `/api/stats`
- **AND** the old `/api/vehicles/{vehicle_id}/stats` and `/api/vehicles/{vehicle_id}/stats/history` paths SHALL no longer supply statistics responses or redirect to a compatibility alias

## REMOVED Requirements

### Requirement: Vehicle summary stats endpoint
**Reason**: Replaced by the combined statistics response; retaining the old route would leave a parallel statistics API.
**Migration**: Request `/api/stats?vehicle_id=<id>` and read the selected-vehicle and fleet summary groups. The old response shape is not retained.

### Requirement: Vehicle stats history endpoint
**Reason**: Complete segment-history delivery is replaced by bounded server-prepared chart series and page-local badge metadata.
**Migration**: Read chart series from `/api/stats?vehicle_id=<id>` and efficiency metadata from each fill-up page. Use the explicit export API when complete raw history is required.

### Requirement: Time-range filtering
**Reason**: The removed summary/history routes owned the optional `from`/`to` contract. The replacement provides exact all-time summaries and explicit bounded calendar windows instead.
**Migration**: Consume the supplied monthly/yearly windows and their range metadata. Arbitrary `from`/`to` filters are not part of the replacement endpoint; clients requiring raw records can use export.
