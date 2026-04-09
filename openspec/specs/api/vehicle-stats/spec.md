### Requirement: Vehicle summary stats endpoint

The system SHALL expose `GET /api/vehicles/{vehicle_id}/stats` returning a JSON object with aggregate fuel efficiency and cost metrics for the specified vehicle. The endpoint SHALL return `404` with code `VEHICLE_NOT_FOUND` if the vehicle does not exist.

The response body SHALL have the following shape:

```json
{
  "total_distance": 1500.0,
  "total_fuel": 120.5,
  "total_cost": 250.00,
  "fill_up_count": 10,
  "average_efficiency": 12.45,
  "average_cost_per_distance": 0.17,
  "distance_unit": "km",
  "volume_unit": "l",
  "currency": "USD"
}
```

All numeric values SHALL be rounded to two decimal places. The `distance_unit`, `volume_unit`, and `currency` fields SHALL reflect the user's current settings. If no fill-ups exist, `total_distance`, `total_fuel`, `total_cost`, and `fill_up_count` SHALL be `0` and `average_efficiency` and `average_cost_per_distance` SHALL be `null`.

#### Scenario: Vehicle with multiple full-tank fill-ups

- **WHEN** the vehicle has 3+ fill-ups with `is_full_tank = true` and no missed fills
- **THEN** the response SHALL include `average_efficiency` computed as the mean of all valid segment efficiencies, and `total_distance` as the difference between the last and first odometer readings

#### Scenario: Vehicle with no fill-ups

- **WHEN** the vehicle exists but has no fill-ups
- **THEN** the response SHALL return `total_distance: 0`, `total_fuel: 0`, `total_cost: 0`, `fill_up_count: 0`, `average_efficiency: null`, `average_cost_per_distance: null`

#### Scenario: Vehicle does not exist

- **WHEN** a request is made with a non-existent `vehicle_id`
- **THEN** the system SHALL return `404` with `{ "code": "VEHICLE_NOT_FOUND", "message": "Vehicle not found." }`

#### Scenario: Only one fill-up exists

- **WHEN** the vehicle has exactly one fill-up
- **THEN** `total_fuel` and `total_cost` SHALL reflect that single fill-up, `total_distance` SHALL be `0`, and `average_efficiency` SHALL be `null` (no segment can be computed from a single fill-up)

### Requirement: Vehicle stats history endpoint

The system SHALL expose `GET /api/vehicles/{vehicle_id}/stats/history` returning a JSON array of per-segment data points ordered chronologically. Each segment represents the interval between two consecutive full-tank fill-ups. The endpoint SHALL return `404` with code `VEHICLE_NOT_FOUND` if the vehicle does not exist.

Each element in the array SHALL have the following shape:

```json
{
  "start_date": "2024-01-15",
  "end_date": "2024-02-01",
  "start_odometer": 10000.0,
  "end_odometer": 10500.0,
  "distance": 500.0,
  "fuel": 42.5,
  "cost": 85.00,
  "efficiency": 11.76,
  "cost_per_distance": 0.17,
  "is_valid": true,
  "distance_unit": "km",
  "volume_unit": "l",
  "currency": "USD"
}
```

The `is_valid` field SHALL be `false` if any fill-up within the segment has `is_missed = true`, indicating the efficiency value is unreliable. Invalid segments SHALL still be included in the response with their computed values, but flagged accordingly.

#### Scenario: Vehicle with valid segments

- **WHEN** the vehicle has 4 full-tank fill-ups with no missed fills between them
- **THEN** the response SHALL contain 3 segment objects ordered by `start_date` ascending, each with `is_valid: true`

#### Scenario: Segment contains a missed fill-up

- **WHEN** a fill-up between two full-tank fills has `is_missed = true`
- **THEN** that segment SHALL have `is_valid: false` and efficiency/cost_per_distance SHALL still be computed from the recorded data

#### Scenario: No segments computable

- **WHEN** the vehicle has fewer than 2 full-tank fill-ups
- **THEN** the response SHALL be an empty array `[]`

#### Scenario: Partial-tank fills between full-tank fills

- **WHEN** there are partial fills (`is_full_tank = false`) between two full-tank fills
- **THEN** the segment SHALL accumulate the fuel and cost from all intermediate fills (including the ending full-tank fill, excluding the starting full-tank fill) and compute efficiency over the total distance

### Requirement: Segment calculation algorithm

The system SHALL compute fuel efficiency segments using the tank-to-tank method:

1. Query fill-ups for the vehicle ordered by `date ASC, id ASC`.
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

### Requirement: Time-range filtering

Both stats endpoints SHALL accept optional `from` and `to` query parameters as ISO date strings (e.g., `2024-01-01`). When provided, only fill-ups with `date >= from` and/or `date <= to` SHALL be included in calculations.

#### Scenario: Filter by date range

- **WHEN** `GET /api/vehicles/1/stats?from=2024-01-01&to=2024-06-30` is requested
- **THEN** only fill-ups with dates between `2024-01-01` and `2024-06-30` inclusive SHALL be used for computation

#### Scenario: No date filter

- **WHEN** `from` and `to` are both omitted
- **THEN** all fill-ups for the vehicle SHALL be included (all-time stats)

#### Scenario: Only from provided

- **WHEN** `from=2024-06-01` is provided without `to`
- **THEN** all fill-ups from `2024-06-01` onward SHALL be included

#### Scenario: Invalid date format

- **WHEN** `from` or `to` contains a non-date string (e.g., `from=banana`)
- **THEN** the system SHALL return `400` with code `STATS_INVALID_DATE_FILTER`

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
