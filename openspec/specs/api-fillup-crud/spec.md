## Purpose

CRUD API for vehicle fill-ups: list, get, create, update (full replace), and delete endpoints, with validation for date, fuel amount, odometer, and cost, a defined response shape, default values, and auto-populated fuel_unit and currency.

## Requirements

### Requirement: List fill-ups for a vehicle

The API SHALL return a cursor-paginated page of fill-ups for a vehicle, ordered by date descending and ID descending.

#### Scenario: First page
- **WHEN** `GET /api/vehicles/{vehicle_id}/fillups` is requested without a cursor for an existing vehicle
- **THEN** the response SHALL be `200 OK`
- **AND** `items` SHALL contain at most 25 fill-ups by default
- **AND** `next_cursor` SHALL be an opaque string when older entries exist or `null` otherwise

#### Scenario: Requested page size
- **WHEN** a valid integer `limit` from 1 through 100 is supplied
- **THEN** `items` SHALL contain at most that many fill-ups

#### Scenario: Continue listing
- **WHEN** a returned cursor is supplied to the next request
- **THEN** the response SHALL contain the next older items in the same order
- **AND** no item from the preceding page SHALL be repeated

#### Scenario: Empty or terminal page
- **WHEN** no additional fill-ups exist
- **THEN** `items` SHALL be empty or contain the remaining items
- **AND** `next_cursor` SHALL be `null`

#### Scenario: Invalid pagination input
- **WHEN** `limit` is invalid
- **THEN** the API SHALL return `400 Bad Request` with code `FILLUP_INVALID_PAGE_LIMIT`
- **WHEN** `cursor` is malformed
- **THEN** the API SHALL return `400 Bad Request` with code `FILLUP_INVALID_CURSOR`

#### Scenario: List for non-existent vehicle
- **WHEN** the list is requested for a vehicle that does not exist
- **THEN** the response SHALL be `404 Not Found`
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

The API SHALL create a new fill-up for a vehicle and return it with a generated ID and timestamps. The `odometer` and `cost` fields are now required. The `fuel_unit` and `currency` fields are auto-populated from the application settings. If supplied in the request body, they SHALL be ignored.

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

### Requirement: Fill-up odometer validation

The API SHALL require a valid odometer value for every fill-up. Odometer readings SHALL NOT decrease across fill-ups for the same vehicle. When updating a fill-up, the reading SHALL fit between the immediately previous and next fill-ups in the existing chronological order, excluding the fill-up being updated. Fill-ups on the same date SHALL be ordered by ID.

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
- **AND** the odometer is greater than or equal to the immediately previous chronological fill-up when one exists
- **AND** the odometer is less than or equal to the immediately next chronological fill-up when one exists
- **THEN** the request SHALL be accepted

#### Scenario: Update with invalid odometer

- **WHEN** a `PUT /api/vehicles/{vehicle_id}/fillups/{id}` request is received with an `odometer` value
- **AND** the odometer is lower than the immediately previous chronological fill-up or higher than the immediately next chronological fill-up
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

### Requirement: Fill-up response shape

All fill-up API responses SHALL use a consistent JSON shape.

#### Scenario: Fill-up JSON structure
- **WHEN** a fill-up is returned in any endpoint response
- **THEN** the JSON object SHALL contain exactly these fields: `id` (integer), `vehicle_id` (integer), `date` (string), `odometer` (number or null), `fuel_amount` (number), `fuel_unit` (string), `cost` (number or null), `currency` (string or null), `is_full_tank` (boolean), `is_missed` (boolean), `station` (string or null), `notes` (string or null), `created_at` (string), `updated_at` (string)

### Requirement: Fill-up default values

The API SHALL apply updated default values for optional boolean fields.

#### Scenario: Default is_full_tank

- **WHEN** a create request omits `is_full_tank` or sets it to `null`
- **THEN** `is_full_tank` SHALL default to `true`

#### Scenario: Default is_missed

- **WHEN** a create request omits `is_missed` or sets it to `null`
- **THEN** `is_missed` SHALL default to `false`

### Requirement: Auto-populated fuel_unit and currency

The API SHALL read `fuel_unit` and `currency` from the application settings table and apply them to every fill-up on create and update. Values supplied for these fields in the request body SHALL be ignored and SHALL NOT override the settings values.

#### Scenario: fuel_unit from settings

- **WHEN** a fill-up is created or updated
- **THEN** `fuel_unit` SHALL be set to the current `volume_unit` value from the settings table (e.g., `"l"` or `"gal"`)

#### Scenario: currency from settings

- **WHEN** a fill-up is created or updated
- **THEN** `currency` SHALL be set to the current `currency` value from the settings table (e.g., `"USD"` or `"EUR"`)

#### Scenario: Request values do not override settings

- **WHEN** a create or update request supplies `fuel_unit` or `currency` values that differ from the application settings
- **THEN** the request SHALL be processed normally
- **AND** the supplied values SHALL be ignored
- **AND** the stored `fuel_unit` and `currency` SHALL use the application settings values
