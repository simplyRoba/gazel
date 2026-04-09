## MODIFIED Requirements

### Requirement: Create a fill-up

The API SHALL create a new fill-up for a vehicle and return it with a generated ID and timestamps. The `odometer` and `cost` fields are now required. The `fuel_unit` and `currency` fields are auto-populated from the application settings and SHALL NOT be accepted in the request body.

#### Scenario: Create with all fields

- **WHEN** a `POST /api/vehicles/{vehicle_id}/fillups` request is received with `date`, `odometer`, `fuel_amount`, `cost`, `is_full_tank`, `is_missed`, `station`, and `notes`
- **AND** the vehicle exists
- **THEN** the response status SHALL be `201 Created`
- **AND** the body SHALL be the created fill-up with a generated `id`, `vehicle_id`, `created_at`, and `updated_at`
- **AND** `fuel_unit` SHALL be set from the application settings `volume_unit` value
- **AND** `currency` SHALL be set from the application settings `currency` value

#### Scenario: Create with only required fields

- **WHEN** a `POST /api/vehicles/{vehicle_id}/fillups` request is received with only `date`, `odometer`, `fuel_amount`, and `cost`
- **AND** the vehicle exists
- **THEN** the response status SHALL be `201 Created`
- **AND** `is_full_tank` SHALL default to `true`
- **AND** `is_missed` SHALL default to `false`
- **AND** `station` and `notes` SHALL be `null`
- **AND** `fuel_unit` and `currency` SHALL be populated from application settings

#### Scenario: Create for non-existent vehicle

- **WHEN** a `POST /api/vehicles/{vehicle_id}/fillups` request is received
- **AND** the vehicle does not exist
- **THEN** the response status SHALL be `404 Not Found`
- **AND** the body SHALL contain `"code": "VEHICLE_NOT_FOUND"`

### Requirement: Update a fill-up (full replace)

The API SHALL fully replace a fill-up's fields via PUT. The `odometer` and `cost` fields are now required. The `fuel_unit` and `currency` are auto-populated from settings.

#### Scenario: Successful update

- **WHEN** a `PUT /api/vehicles/{vehicle_id}/fillups/{id}` request is received with valid fields including `date`, `odometer`, `fuel_amount`, and `cost`
- **AND** the vehicle exists
- **AND** the fill-up exists and belongs to that vehicle
- **THEN** the response status SHALL be `200 OK`
- **AND** the body SHALL be the updated fill-up
- **AND** `updated_at` SHALL be set to the current time
- **AND** `fuel_unit` SHALL be set from the application settings `volume_unit` value
- **AND** `currency` SHALL be set from the application settings `currency` value

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

### Requirement: Fill-up odometer validation

The API SHALL require a valid odometer value for every fill-up. Odometer readings SHALL NOT decrease across fill-ups for the same vehicle.

#### Scenario: Missing odometer on create

- **WHEN** a `POST /api/vehicles/{vehicle_id}/fillups` request is received without an `odometer` field or with `odometer` set to `null`
- **THEN** the response status SHALL be `422 Unprocessable Entity`
- **AND** the body SHALL contain `"code": "FILLUP_ODOMETER_REQUIRED"`

#### Scenario: Create with valid odometer (higher than previous)

- **WHEN** a `POST /api/vehicles/{vehicle_id}/fillups` request is received with an `odometer` value
- **AND** the value is greater than or equal to the highest existing odometer for that vehicle
- **THEN** the request SHALL be accepted

#### Scenario: Create with invalid odometer (lower than previous)

- **WHEN** a `POST /api/vehicles/{vehicle_id}/fillups` request is received with an `odometer` value
- **AND** the value is less than the highest existing odometer for that vehicle
- **THEN** the response status SHALL be `422 Unprocessable Entity`
- **AND** the body SHALL contain `"code": "FILLUP_INVALID_ODOMETER"`

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

The API SHALL require a non-negative cost value for every fill-up.

#### Scenario: Missing cost on create

- **WHEN** a `POST /api/vehicles/{vehicle_id}/fillups` request is received without a `cost` field or with `cost` set to `null`
- **THEN** the response status SHALL be `422 Unprocessable Entity`
- **AND** the body SHALL contain `"code": "FILLUP_COST_REQUIRED"`

#### Scenario: Negative cost

- **WHEN** a create or update request includes a `cost` less than `0`
- **THEN** the response status SHALL be `422 Unprocessable Entity`
- **AND** the body SHALL contain `"code": "FILLUP_INVALID_COST"`

#### Scenario: Zero cost is valid

- **WHEN** a create or update request includes a `cost` of `0`
- **THEN** the request SHALL be accepted

### Requirement: Fill-up default values

The API SHALL apply updated default values for optional boolean fields.

#### Scenario: Default is_full_tank

- **WHEN** a create request omits `is_full_tank` or sets it to `null`
- **THEN** `is_full_tank` SHALL default to `true`

#### Scenario: Default is_missed

- **WHEN** a create request omits `is_missed` or sets it to `null`
- **THEN** `is_missed` SHALL default to `false`

### Requirement: Auto-populated fuel_unit and currency

The API SHALL read `fuel_unit` and `currency` from the application settings table and apply them to every fill-up on create and update. These fields SHALL NOT be accepted from the request body.

#### Scenario: fuel_unit from settings

- **WHEN** a fill-up is created or updated
- **THEN** `fuel_unit` SHALL be set to the current `volume_unit` value from the settings table (e.g., `"l"` or `"gal"`)

#### Scenario: currency from settings

- **WHEN** a fill-up is created or updated
- **THEN** `currency` SHALL be set to the current `currency` value from the settings table (e.g., `"USD"` or `"EUR"`)

## REMOVED Requirements

### Requirement: Create with null odometer accepted

**Reason**: Odometer is now required to enable efficiency calculations in a future change.
**Migration**: All create/update requests must include a numeric `odometer` value.

### Requirement: Create with null cost accepted

**Reason**: Cost is now required to enable cost tracking, which is a core feature.
**Migration**: All create/update requests must include a numeric `cost` value (use `0` if cost is truly unknown).

### Requirement: Odometer ordering validation - null odometer scenario

**Reason**: Null odometer is no longer accepted; this scenario is superseded by the `FILLUP_ODOMETER_REQUIRED` validation.
**Migration**: N/A -- the scenario simply no longer applies.
