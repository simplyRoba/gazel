## ADDED Requirements

### Requirement: Vehicle list state

The vehicle store SHALL maintain a reactive list of all vehicles, a loading flag, and an error state.

#### Scenario: Initial state
- **WHEN** the store is first accessed
- **THEN** the vehicle list SHALL be empty
- **AND** loading SHALL be `false`
- **AND** error SHALL be `null`

### Requirement: Load vehicles action

The store SHALL provide a `loadVehicles()` action that fetches all vehicles from the API.

#### Scenario: Successful load
- **WHEN** `loadVehicles()` is called and the API returns vehicles
- **THEN** the vehicle list SHALL be set to the returned data
- **AND** error SHALL be `null`

#### Scenario: Load failure
- **WHEN** `loadVehicles()` is called and the API throws an error
- **THEN** the vehicle list SHALL remain unchanged
- **AND** error SHALL be set to the error message

### Requirement: Create vehicle action

The store SHALL provide a `createVehicle()` action that creates a vehicle and appends it to the list.

#### Scenario: Successful create
- **WHEN** `createVehicle(data)` is called and the API succeeds
- **THEN** the new vehicle SHALL be appended to the list
- **AND** the function SHALL return the created vehicle

#### Scenario: Create failure
- **WHEN** `createVehicle(data)` is called and the API throws an error
- **THEN** the list SHALL remain unchanged
- **AND** error SHALL be set to the error message
- **AND** the function SHALL return `null`

### Requirement: Update vehicle action

The store SHALL provide an `updateVehicle()` action that updates a vehicle in place.

#### Scenario: Successful update
- **WHEN** `updateVehicle(id, data)` is called and the API succeeds
- **THEN** the matching vehicle in the list SHALL be replaced with the updated version
- **AND** the function SHALL return the updated vehicle

#### Scenario: Update failure
- **WHEN** `updateVehicle(id, data)` is called and the API throws an error
- **THEN** the list SHALL remain unchanged
- **AND** error SHALL be set to the error message
- **AND** the function SHALL return `null`

### Requirement: Delete vehicle action

The store SHALL provide a `deleteVehicle()` action that removes a vehicle from the list.

#### Scenario: Successful delete
- **WHEN** `deleteVehicle(id)` is called and the API succeeds
- **THEN** the vehicle SHALL be removed from the list
- **AND** the function SHALL return `true`

#### Scenario: Delete failure
- **WHEN** `deleteVehicle(id)` is called and the API throws an error
- **THEN** the list SHALL remain unchanged
- **AND** error SHALL be set to the error message
- **AND** the function SHALL return `false`

### Requirement: Error clearing

Every store action SHALL clear the previous error before making an API call.

#### Scenario: Error is cleared on new action
- **WHEN** any store action is called
- **THEN** the error state SHALL be set to `null` before the API call is made
