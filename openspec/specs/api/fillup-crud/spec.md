## ADDED Requirements

### Requirement: List fill-ups for a vehicle

The API SHALL return all fill-ups for a given vehicle, sorted by date descending.

#### Scenario: List when fill-ups exist
- **WHEN** a `GET /api/vehicles/{vehicle_id}/fillups` request is received
- **AND** the vehicle exists
- **AND** fill-ups exist for that vehicle
- **THEN** the response status SHALL be `200 OK`
- **AND** the body SHALL be a JSON array of fill-up objects sorted by date descending

#### Scenario: List when no fill-ups exist
- **WHEN** a `GET /api/vehicles/{vehicle_id}/fillups` request is received
- **AND** the vehicle exists
- **AND** no fill-ups exist for that vehicle
- **THEN** the response status SHALL be `200 OK`
- **AND** the body SHALL be an empty JSON array `[]`

#### Scenario: List for non-existent vehicle
- **WHEN** a `GET /api/vehicles/{vehicle_id}/fillups` request is received
- **AND** the vehicle does not exist
- **THEN** the response status SHALL be `404 Not Found`
- **AND** the body SHALL contain `"code": "VEHICLE_NOT_FOUND"`

### Requirement: Get a single fill-up

The API SHALL return a single fill-up by its ID, scoped to a vehicle.

#### Scenario: Fill-up exists
- **WHEN** a `GET /api/vehicles/{vehicle_id}/fillups/{id}` request is received
- **AND** the vehicle exists
- **AND** the fill-up exists and belongs to that vehicle
- **THEN** the response status SHALL be `200 OK`
- **AND** the body SHALL be a JSON fill-up object

#### Scenario: Fill-up not found
- **WHEN** a `GET /api/vehicles/{vehicle_id}/fillups/{id}` request is received
- **AND** the vehicle exists
- **AND** no fill-up with that ID exists for the vehicle
- **THEN** the response status SHALL be `404 Not Found`
- **AND** the body SHALL contain `"code": "FILLUP_NOT_FOUND"`

#### Scenario: Vehicle not found for get
- **WHEN** a `GET /api/vehicles/{vehicle_id}/fillups/{id}` request is received
- **AND** the vehicle does not exist
- **THEN** the response status SHALL be `404 Not Found`
- **AND** the body SHALL contain `"code": "VEHICLE_NOT_FOUND"`

### Requirement: Create a fill-up

The API SHALL create a new fill-up for a vehicle and return it with a generated ID and timestamps.

#### Scenario: Create with all fields
- **WHEN** a `POST /api/vehicles/{vehicle_id}/fillups` request is received with `date`, `odometer`, `fuel_amount`, `fuel_unit`, `cost`, `currency`, `is_full_tank`, `is_missed`, `station`, and `notes`
- **AND** the vehicle exists
- **THEN** the response status SHALL be `201 Created`
- **AND** the body SHALL be the created fill-up with a generated `id`, `vehicle_id`, `created_at`, and `updated_at`

#### Scenario: Create with only required fields
- **WHEN** a `POST /api/vehicles/{vehicle_id}/fillups` request is received with only `date` and `fuel_amount`
- **AND** the vehicle exists
- **THEN** the response status SHALL be `201 Created`
- **AND** `fuel_unit` SHALL default to `"liters"`
- **AND** `is_full_tank` SHALL default to `false`
- **AND** `is_missed` SHALL default to `false`
- **AND** `odometer`, `cost`, `currency`, `station`, and `notes` SHALL be `null`

#### Scenario: Create for non-existent vehicle
- **WHEN** a `POST /api/vehicles/{vehicle_id}/fillups` request is received
- **AND** the vehicle does not exist
- **THEN** the response status SHALL be `404 Not Found`
- **AND** the body SHALL contain `"code": "VEHICLE_NOT_FOUND"`

### Requirement: Update a fill-up (full replace)

The API SHALL fully replace a fill-up's fields via PUT.

#### Scenario: Successful update
- **WHEN** a `PUT /api/vehicles/{vehicle_id}/fillups/{id}` request is received with valid fields
- **AND** the vehicle exists
- **AND** the fill-up exists and belongs to that vehicle
- **THEN** the response status SHALL be `200 OK`
- **AND** the body SHALL be the updated fill-up
- **AND** `updated_at` SHALL be set to the current time

#### Scenario: Update non-existent fill-up
- **WHEN** a `PUT /api/vehicles/{vehicle_id}/fillups/{id}` request is received
- **AND** the vehicle exists
- **AND** no fill-up with that ID exists for the vehicle
- **THEN** the response status SHALL be `404 Not Found`
- **AND** the body SHALL contain `"code": "FILLUP_NOT_FOUND"`

#### Scenario: Update for non-existent vehicle
- **WHEN** a `PUT /api/vehicles/{vehicle_id}/fillups/{id}` request is received
- **AND** the vehicle does not exist
- **THEN** the response status SHALL be `404 Not Found`
- **AND** the body SHALL contain `"code": "VEHICLE_NOT_FOUND"`

### Requirement: Delete a fill-up

The API SHALL delete a fill-up by ID, returning 204 on success.

#### Scenario: Successful delete
- **WHEN** a `DELETE /api/vehicles/{vehicle_id}/fillups/{id}` request is received
- **AND** the vehicle exists
- **AND** the fill-up exists and belongs to that vehicle
- **THEN** the response status SHALL be `204 No Content`
- **AND** subsequent `GET /api/vehicles/{vehicle_id}/fillups/{id}` SHALL return `404 Not Found`

#### Scenario: Delete non-existent fill-up
- **WHEN** a `DELETE /api/vehicles/{vehicle_id}/fillups/{id}` request is received
- **AND** the vehicle exists
- **AND** no fill-up with that ID exists for the vehicle
- **THEN** the response status SHALL be `404 Not Found`
- **AND** the body SHALL contain `"code": "FILLUP_NOT_FOUND"`

#### Scenario: Delete for non-existent vehicle
- **WHEN** a `DELETE /api/vehicles/{vehicle_id}/fillups/{id}` request is received
- **AND** the vehicle does not exist
- **THEN** the response status SHALL be `404 Not Found`
- **AND** the body SHALL contain `"code": "VEHICLE_NOT_FOUND"`

### Requirement: Fill-up date validation

The API SHALL require a valid date for every fill-up.

#### Scenario: Missing date on create
- **WHEN** a `POST /api/vehicles/{vehicle_id}/fillups` request is received without a `date` field
- **THEN** the response status SHALL be `422 Unprocessable Entity`
- **AND** the body SHALL contain `"code": "FILLUP_DATE_REQUIRED"`

#### Scenario: Empty date on create
- **WHEN** a `POST /api/vehicles/{vehicle_id}/fillups` request is received with a `date` that is empty or whitespace-only
- **THEN** the response status SHALL be `422 Unprocessable Entity`
- **AND** the body SHALL contain `"code": "FILLUP_DATE_REQUIRED"`

#### Scenario: Date is trimmed
- **WHEN** a create or update request includes a `date` with leading/trailing whitespace
- **THEN** the stored date SHALL have whitespace trimmed

### Requirement: Fill-up fuel amount validation

The API SHALL require a positive fuel amount for every fill-up.

#### Scenario: Missing fuel amount on create
- **WHEN** a `POST /api/vehicles/{vehicle_id}/fillups` request is received without a `fuel_amount` field
- **THEN** the response status SHALL be `422 Unprocessable Entity`
- **AND** the body SHALL contain `"code": "FILLUP_FUEL_AMOUNT_REQUIRED"`

#### Scenario: Zero fuel amount
- **WHEN** a create or update request includes `fuel_amount` of `0` or less
- **THEN** the response status SHALL be `422 Unprocessable Entity`
- **AND** the body SHALL contain `"code": "FILLUP_INVALID_FUEL_AMOUNT"`

### Requirement: Odometer ordering validation

The API SHALL validate that odometer readings do not decrease across fill-ups for the same vehicle.

#### Scenario: Create with valid odometer (higher than previous)
- **WHEN** a `POST /api/vehicles/{vehicle_id}/fillups` request is received with an `odometer` value
- **AND** the value is greater than or equal to the highest existing odometer for that vehicle
- **THEN** the request SHALL be accepted

#### Scenario: Create with invalid odometer (lower than previous)
- **WHEN** a `POST /api/vehicles/{vehicle_id}/fillups` request is received with an `odometer` value
- **AND** the value is less than the highest existing odometer for that vehicle
- **THEN** the response status SHALL be `422 Unprocessable Entity`
- **AND** the body SHALL contain `"code": "FILLUP_INVALID_ODOMETER"`

#### Scenario: Create with null odometer
- **WHEN** a create request omits `odometer` or sets it to `null`
- **THEN** no odometer validation SHALL be performed
- **AND** the request SHALL be accepted

#### Scenario: Update with valid odometer
- **WHEN** a `PUT /api/vehicles/{vehicle_id}/fillups/{id}` request is received with an `odometer` value
- **AND** the value is greater than or equal to the highest existing odometer for that vehicle, excluding the fill-up being updated
- **THEN** the request SHALL be accepted

#### Scenario: Update with invalid odometer
- **WHEN** a `PUT /api/vehicles/{vehicle_id}/fillups/{id}` request is received with an `odometer` value
- **AND** the value is less than the highest existing odometer for that vehicle, excluding the fill-up being updated
- **THEN** the response status SHALL be `422 Unprocessable Entity`
- **AND** the body SHALL contain `"code": "FILLUP_INVALID_ODOMETER"`

### Requirement: Fill-up cost validation

The API SHALL validate that cost, when provided, is non-negative.

#### Scenario: Negative cost
- **WHEN** a create or update request includes a `cost` less than `0`
- **THEN** the response status SHALL be `422 Unprocessable Entity`
- **AND** the body SHALL contain `"code": "FILLUP_INVALID_COST"`

#### Scenario: Null cost is valid
- **WHEN** a create or update request omits `cost` or sets it to `null`
- **THEN** the request SHALL be accepted with `cost` set to `null`

### Requirement: Fill-up response shape

All fill-up API responses SHALL use a consistent JSON shape.

#### Scenario: Fill-up JSON structure
- **WHEN** a fill-up is returned in any endpoint response
- **THEN** the JSON object SHALL contain exactly these fields: `id` (integer), `vehicle_id` (integer), `date` (string), `odometer` (number or null), `fuel_amount` (number), `fuel_unit` (string), `cost` (number or null), `currency` (string or null), `is_full_tank` (boolean), `is_missed` (boolean), `station` (string or null), `notes` (string or null), `created_at` (string), `updated_at` (string)
