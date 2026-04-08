## ADDED Requirements

### Requirement: List all vehicles

The API SHALL return all vehicles ordered by name.

#### Scenario: List when vehicles exist
- **WHEN** a `GET /api/vehicles` request is received
- **AND** vehicles exist in the database
- **THEN** the response status SHALL be `200 OK`
- **AND** the body SHALL be a JSON array of vehicle objects ordered by name ascending

#### Scenario: List when no vehicles exist
- **WHEN** a `GET /api/vehicles` request is received
- **AND** no vehicles exist in the database
- **THEN** the response status SHALL be `200 OK`
- **AND** the body SHALL be an empty JSON array `[]`

### Requirement: Get a single vehicle

The API SHALL return a single vehicle by its ID.

#### Scenario: Vehicle exists
- **WHEN** a `GET /api/vehicles/:id` request is received with a valid ID
- **THEN** the response status SHALL be `200 OK`
- **AND** the body SHALL be a JSON object with fields: `id`, `name`, `make`, `model`, `year`, `fuel_type`, `notes`, `created_at`, `updated_at`

#### Scenario: Vehicle not found
- **WHEN** a `GET /api/vehicles/:id` request is received with an unknown ID
- **THEN** the response status SHALL be `404 Not Found`
- **AND** the body SHALL contain `"code": "VEHICLE_NOT_FOUND"`

### Requirement: Create a vehicle

The API SHALL create a new vehicle and return it with a generated ID and timestamps.

#### Scenario: Create with all fields
- **WHEN** a `POST /api/vehicles` request is received with `name`, `make`, `model`, `year`, `fuel_type`, and `notes`
- **THEN** the response status SHALL be `201 Created`
- **AND** the body SHALL be the created vehicle with a generated `id`, `created_at`, and `updated_at`

#### Scenario: Create with only required fields
- **WHEN** a `POST /api/vehicles` request is received with only `name`
- **THEN** the response status SHALL be `201 Created`
- **AND** `fuel_type` SHALL default to `"gasoline"`
- **AND** `make`, `model`, `year`, and `notes` SHALL be `null`

#### Scenario: Create with missing name
- **WHEN** a `POST /api/vehicles` request is received without a `name` field
- **THEN** the response status SHALL be `422 Unprocessable Entity`
- **AND** the body SHALL contain `"code": "VEHICLE_NAME_REQUIRED"`

#### Scenario: Create with empty name
- **WHEN** a `POST /api/vehicles` request is received with a `name` that is empty or whitespace-only
- **THEN** the response status SHALL be `422 Unprocessable Entity`
- **AND** the body SHALL contain `"code": "VEHICLE_NAME_REQUIRED"`

#### Scenario: Name is trimmed
- **WHEN** a `POST /api/vehicles` request is received with a `name` containing leading/trailing whitespace
- **THEN** the stored name SHALL have whitespace trimmed

### Requirement: Update a vehicle (full replace)

The API SHALL fully replace a vehicle's fields via PUT.

#### Scenario: Successful update
- **WHEN** a `PUT /api/vehicles/:id` request is received with valid fields
- **THEN** the response status SHALL be `200 OK`
- **AND** the body SHALL be the updated vehicle
- **AND** `updated_at` SHALL be set to the current time

#### Scenario: Update non-existent vehicle
- **WHEN** a `PUT /api/vehicles/:id` request is received with an unknown ID
- **THEN** the response status SHALL be `404 Not Found`
- **AND** the body SHALL contain `"code": "VEHICLE_NOT_FOUND"`

### Requirement: Patch a vehicle (partial update)

The API SHALL partially update a vehicle's fields via PATCH, using three-state semantics for nullable fields.

#### Scenario: Update only sent fields
- **WHEN** a `PATCH /api/vehicles/:id` request is received with a subset of fields
- **THEN** only the sent fields SHALL be updated
- **AND** fields not included in the request SHALL retain their current values

#### Scenario: Clear a nullable field
- **WHEN** a `PATCH /api/vehicles/:id` request includes a field set to `null` (e.g., `"notes": null`)
- **THEN** that field SHALL be set to `NULL` in the database

#### Scenario: Absent field is preserved
- **WHEN** a `PATCH /api/vehicles/:id` request omits a field entirely
- **THEN** that field SHALL retain its current value

#### Scenario: Patch non-existent vehicle
- **WHEN** a `PATCH /api/vehicles/:id` request is received with an unknown ID
- **THEN** the response status SHALL be `404 Not Found`
- **AND** the body SHALL contain `"code": "VEHICLE_NOT_FOUND"`

### Requirement: Delete a vehicle

The API SHALL delete a vehicle by ID, returning 204 on success.

#### Scenario: Successful delete
- **WHEN** a `DELETE /api/vehicles/:id` request is received for an existing vehicle with no fill-ups
- **THEN** the response status SHALL be `204 No Content`
- **AND** subsequent `GET /api/vehicles/:id` SHALL return `404 Not Found`

#### Scenario: Delete non-existent vehicle
- **WHEN** a `DELETE /api/vehicles/:id` request is received with an unknown ID
- **THEN** the response status SHALL be `404 Not Found`
- **AND** the body SHALL contain `"code": "VEHICLE_NOT_FOUND"`

### Requirement: Fuel type validation

The API SHALL validate that `fuel_type` is one of the allowed values.

#### Scenario: Valid fuel type
- **WHEN** a create or update request includes a `fuel_type` from the set `gasoline`, `diesel`, `electric`, `hybrid`, `lpg`, `cng`, `other`
- **THEN** the request SHALL be accepted

#### Scenario: Invalid fuel type
- **WHEN** a create or update request includes a `fuel_type` not in the allowed set
- **THEN** the response status SHALL be `422 Unprocessable Entity`
- **AND** the body SHALL contain `"code": "VEHICLE_INVALID_FUEL_TYPE"`

### Requirement: Year validation

The API SHALL validate that `year`, when provided, is within a reasonable range.

#### Scenario: Valid year
- **WHEN** a create or update request includes a `year` between 1900 and 2100
- **THEN** the request SHALL be accepted

#### Scenario: Invalid year
- **WHEN** a create or update request includes a `year` outside the 1900-2100 range
- **THEN** the response status SHALL be `422 Unprocessable Entity`
- **AND** the body SHALL contain `"code": "VEHICLE_INVALID_YEAR"`

#### Scenario: Null year is valid
- **WHEN** a create or update request includes `"year": null` or omits `year`
- **THEN** the request SHALL be accepted with `year` set to or remaining `null`

### Requirement: Vehicle response shape

All vehicle API responses SHALL use a consistent JSON shape.

#### Scenario: Vehicle JSON structure
- **WHEN** a vehicle is returned in any endpoint response
- **THEN** the JSON object SHALL contain exactly these fields: `id` (integer), `name` (string), `make` (string or null), `model` (string or null), `year` (integer or null), `fuel_type` (string), `notes` (string or null), `created_at` (string), `updated_at` (string)
