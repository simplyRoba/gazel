## Purpose

Dashboard statistics UI — stats API client types and functions, reactive stats store with caching and invalidation, fleet summary cards with responsive layout, and per-vehicle stats row.

## Requirements

### Requirement: Stats API client types

The API client SHALL export TypeScript interfaces for vehicle stats responses.

#### Scenario: VehicleStats interface

- **WHEN** a `VehicleStats` object is used in the frontend
- **THEN** it SHALL have fields: `total_distance` (number), `total_fuel` (number), `total_cost` (number), `fill_up_count` (number), `average_efficiency` (number | null), `average_cost_per_distance` (number | null), `distance_unit` (string), `volume_unit` (string), `currency` (string)

#### Scenario: SegmentHistory interface

- **WHEN** a `SegmentHistory` object is used in the frontend
- **THEN** it SHALL have fields: `start_date` (string), `end_date` (string), `start_odometer` (number), `end_odometer` (number), `distance` (number), `fuel` (number), `cost` (number), `efficiency` (number), `cost_per_distance` (number), `is_valid` (boolean), `distance_unit` (string), `volume_unit` (string), `currency` (string)

### Requirement: Stats API client functions

The API client SHALL export typed functions for fetching vehicle stats.

#### Scenario: Fetch vehicle stats

- **WHEN** `fetchVehicleStats(vehicleId)` is called
- **THEN** it SHALL send `GET /api/vehicles/{vehicleId}/stats` and return `VehicleStats`

#### Scenario: Fetch vehicle stats history

- **WHEN** `fetchVehicleStatsHistory(vehicleId)` is called
- **THEN** it SHALL send `GET /api/vehicles/{vehicleId}/stats/history` and return `SegmentHistory[]`

### Requirement: Stats store state

The stats store SHALL maintain a reactive cache of vehicle stats and segment history, keyed by vehicle ID.

#### Scenario: Initial state

- **WHEN** the stats store is first accessed
- **THEN** the stats cache SHALL be empty
- **AND** the history cache SHALL be empty
- **AND** loading SHALL be `false`
- **AND** error SHALL be `null`

#### Scenario: State accessors

- **WHEN** store state is read from a component
- **THEN** it SHALL be accessed via exported getter functions: `getVehicleStats(vehicleId)`, `getVehicleHistory(vehicleId)`, `getLoading()`, `getError()`

### Requirement: Stats store load actions

The stats store SHALL provide actions to fetch and cache stats data.

#### Scenario: Load stats for a vehicle

- **WHEN** `loadStats(vehicleId)` is called and the API succeeds
- **THEN** the `VehicleStats` for that vehicle SHALL be stored in the stats cache
- **AND** the `SegmentHistory[]` for that vehicle SHALL be stored in the history cache
- **AND** error SHALL be `null`

#### Scenario: Load stats failure

- **WHEN** `loadStats(vehicleId)` is called and the API fails
- **THEN** the caches SHALL remain unchanged
- **AND** error SHALL be set to the error message

#### Scenario: Load stats for all vehicles

- **WHEN** `loadAllStats(vehicleIds)` is called
- **THEN** it SHALL fetch stats for every vehicle ID in parallel
- **AND** each vehicle's stats and history SHALL be cached individually

### Requirement: Stats store invalidation

The stats store SHALL provide an invalidation mechanism so stats refresh after fill-up mutations.

#### Scenario: Invalidate single vehicle

- **WHEN** `invalidateStats(vehicleId)` is called
- **THEN** the cached stats and history for that vehicle SHALL be cleared
- **AND** `loadStats(vehicleId)` SHALL be triggered to refetch

### Requirement: Fleet summary cards

The dashboard SHALL display a row of summary cards showing aggregate metrics. For a single vehicle the cards show that vehicle's data; for multiple vehicles the cards aggregate across all vehicles.

#### Scenario: Summary cards displayed

- **WHEN** the dashboard loads and at least one vehicle exists
- **THEN** summary cards SHALL be displayed showing: total distance (formatted with `formatDistance`), total fill-ups count, cost per distance unit (formatted with `formatCurrency` and the user's distance_unit), and fuel price (total cost divided by total fuel, formatted with `formatCurrency` and the user's volume_unit)

#### Scenario: No distance or fuel data

- **WHEN** all vehicles have zero fill-ups (total_distance and total_fuel are 0)
- **THEN** the cost per distance and fuel price cards SHALL show a placeholder "—" instead of a number

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

### Requirement: Per-vehicle stats row

The dashboard SHALL display per-vehicle stats below the chip row when multiple vehicles exist. The stats shown SHALL match the same 4 metrics as the summary cards, scoped to the selected vehicle.

#### Scenario: Stats shown for active vehicle

- **WHEN** a vehicle chip is selected and that vehicle has stats loaded
- **THEN** the dashboard SHALL display below the chip row: total distance, fill-ups count, cost per distance unit, and fuel price — all for the selected vehicle only

#### Scenario: No data available

- **WHEN** the selected vehicle has no distance or fuel data
- **THEN** the cost per distance and fuel price displays SHALL show "—"

#### Scenario: Stats loading

- **WHEN** stats for the selected vehicle are still loading
- **THEN** the per-vehicle stats area SHALL show shimmer/skeleton placeholders

#### Scenario: Single vehicle

- **WHEN** only one vehicle exists
- **THEN** the per-vehicle stats row SHALL NOT be displayed (summary cards already show this vehicle's data)

### Requirement: Stats store error clearing

Every stats store action SHALL clear the previous error before making an API call.

#### Scenario: Error cleared on new action

- **WHEN** any stats store action is called
- **THEN** the error state SHALL be set to `null` before the API call is made
