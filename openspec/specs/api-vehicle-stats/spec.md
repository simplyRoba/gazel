## Purpose

Aggregate fuel-efficiency and cost statistics for a vehicle: a summary stats endpoint, a history endpoint, the segment calculation algorithm, time-range filtering, and unit-system-aware responses.

## Requirements

### Requirement: Segment calculation algorithm

The system SHALL compute fuel efficiency segments using the tank-to-tank method:

1. Evaluate the vehicle's fill-ups in ascending chronological order, using the fill-up API's defined same-date ordering.
2. Skip fill-ups with `odometer <= 0`.
3. Identify consecutive full-tank fill-ups as segment boundaries.
4. For each segment, sum `fuel_amount` and `cost` of all fill-ups after the start boundary through the end boundary (inclusive of end, exclusive of start).
5. Compute `distance = end_odometer - start_odometer`.
6. Compute `efficiency = distance / total_fuel`.
7. Compute `cost_per_distance = total_cost / distance`.
8. Mark segment as invalid if any fill-up in the segment has `is_missed = true`.

#### Scenario: Standard two full-tank fills

- **WHEN** fill-up A has `odometer: 1000, is_full_tank: true` and fill-up B has `odometer: 1500, fuel_amount: 40.0, is_full_tank: true`
- **THEN** the segment efficiency SHALL be `(1500 - 1000) / 40.0 = 12.5`

#### Scenario: Partial fills between full-tank fills

- **WHEN** fill-up A has `odometer: 1000, is_full_tank: true`, fill-up B has `odometer: 1200, fuel_amount: 15.0, is_full_tank: false`, and fill-up C has `odometer: 1500, fuel_amount: 25.0, is_full_tank: true`
- **THEN** the segment fuel SHALL be `15.0 + 25.0 = 40.0` and efficiency SHALL be `(1500 - 1000) / 40.0 = 12.5`

#### Scenario: Zero-odometer fill-ups are excluded

- **WHEN** a fill-up has `odometer: 0` (legacy null-to-default)
- **THEN** that fill-up SHALL be excluded from segment computation entirely

### Requirement: Unit-system-aware responses

All stats responses SHALL express values in the user's currently configured unit system. The system SHALL read `distance_unit` and `volume_unit` from settings and convert computed values accordingly.

When fill-up records were stored with a different `fuel_unit` than the user's current `volume_unit`, the system SHALL normalize fuel amounts to a common base (liters) before computing, then convert the result to the display unit.

#### Scenario: Metric settings

- **WHEN** settings have `distance_unit: "km"` and `volume_unit: "l"`
- **THEN** efficiency SHALL be expressed as km/L, distances in km, and volumes in L

#### Scenario: Imperial settings

- **WHEN** settings have `distance_unit: "mi"` and `volume_unit: "gal"`
- **THEN** efficiency SHALL be expressed as mi/gal (MPG), distances in mi, and volumes in gal

#### Scenario: Mixed historical units

- **WHEN** some fill-ups were recorded with `fuel_unit: "l"` and the current setting is `volume_unit: "gal"`
- **THEN** the system SHALL convert liter amounts to gallons before computing and displaying stats

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
