## MODIFIED Requirements

### Requirement: Delete a vehicle

The API SHALL delete a vehicle by ID, returning 204 on success. The API SHALL reject deletion of a vehicle that has existing fill-ups.

#### Scenario: Successful delete
- **WHEN** a `DELETE /api/vehicles/:id` request is received for an existing vehicle with no fill-ups
- **THEN** the response status SHALL be `204 No Content`
- **AND** subsequent `GET /api/vehicles/:id` SHALL return `404 Not Found`

#### Scenario: Delete vehicle with fill-ups
- **WHEN** a `DELETE /api/vehicles/:id` request is received for an existing vehicle
- **AND** the vehicle has one or more fill-ups
- **THEN** the response status SHALL be `409 Conflict`
- **AND** the body SHALL contain `"code": "VEHICLE_HAS_FILLUPS"`

#### Scenario: Delete non-existent vehicle
- **WHEN** a `DELETE /api/vehicles/:id` request is received with an unknown ID
- **THEN** the response status SHALL be `404 Not Found`
- **AND** the body SHALL contain `"code": "VEHICLE_NOT_FOUND"`
