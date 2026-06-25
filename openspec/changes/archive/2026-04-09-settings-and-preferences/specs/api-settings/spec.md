## ADDED Requirements

### Requirement: Settings table with singleton row

The database SHALL contain a `settings` table with exactly one row (`id = 1`), enforced by a `CHECK (id = 1)` constraint. The migration SHALL seed the row with default values.

#### Scenario: Fresh database after migration

- **WHEN** the migration runs on a new database
- **THEN** the `settings` table SHALL exist with columns: `id`, `unit_system`, `distance_unit`, `volume_unit`, `currency`, `color_mode`, `locale`
- **AND** exactly one row SHALL exist with `id = 1`
- **AND** default values SHALL be: `unit_system = 'metric'`, `distance_unit = 'km'`, `volume_unit = 'l'`, `currency = 'USD'`, `color_mode = 'system'`, `locale = 'en'`

#### Scenario: Singleton constraint prevents additional rows

- **WHEN** an attempt is made to insert a second row into the `settings` table
- **THEN** the database SHALL reject the insert with a constraint violation

### Requirement: Read settings endpoint

`GET /api/settings` SHALL return the current settings as a JSON object.

#### Scenario: Successful read

- **WHEN** a `GET` request is made to `/api/settings`
- **THEN** the response status SHALL be `200`
- **AND** the response body SHALL be a JSON object with keys: `unit_system`, `distance_unit`, `volume_unit`, `currency`, `color_mode`, `locale`

#### Scenario: Database error

- **WHEN** a `GET` request is made to `/api/settings` and the database is unavailable
- **THEN** the response status SHALL be `500`
- **AND** the response body SHALL follow the standard `{ "code": "INTERNAL_ERROR", "message": "..." }` shape

### Requirement: Update settings endpoint

`PUT /api/settings` SHALL accept a partial JSON body and update only the provided fields, leaving omitted fields unchanged.

#### Scenario: Update a single field

- **WHEN** a `PUT` request is made to `/api/settings` with body `{ "color_mode": "dark" }`
- **THEN** the response status SHALL be `200`
- **AND** the response body SHALL contain all settings fields with `color_mode` set to `"dark"` and all other fields unchanged from their previous values

#### Scenario: Update multiple fields

- **WHEN** a `PUT` request is made to `/api/settings` with body `{ "unit_system": "imperial", "distance_unit": "mi", "volume_unit": "gal" }`
- **THEN** the response status SHALL be `200`
- **AND** the response body SHALL reflect all three updated fields

#### Scenario: Empty body

- **WHEN** a `PUT` request is made to `/api/settings` with body `{}`
- **THEN** the response status SHALL be `200`
- **AND** all settings SHALL remain unchanged

### Requirement: Settings field validation

The API SHALL validate field values and reject invalid inputs with a `422` status.

#### Scenario: Invalid color_mode

- **WHEN** a `PUT` request is made with `{ "color_mode": "purple" }`
- **THEN** the response status SHALL be `422`
- **AND** the response body SHALL contain `{ "code": "SETTINGS_INVALID_COLOR_MODE", "message": "..." }`

#### Scenario: Invalid unit_system

- **WHEN** a `PUT` request is made with `{ "unit_system": "martian" }`
- **THEN** the response status SHALL be `422`
- **AND** the response body SHALL contain `{ "code": "SETTINGS_INVALID_UNIT_SYSTEM", "message": "..." }`

#### Scenario: Invalid distance_unit

- **WHEN** a `PUT` request is made with `{ "distance_unit": "parsec" }`
- **THEN** the response status SHALL be `422`
- **AND** the response body SHALL contain `{ "code": "SETTINGS_INVALID_DISTANCE_UNIT", "message": "..." }`

#### Scenario: Invalid volume_unit

- **WHEN** a `PUT` request is made with `{ "volume_unit": "barrel" }`
- **THEN** the response status SHALL be `422`
- **AND** the response body SHALL contain `{ "code": "SETTINGS_INVALID_VOLUME_UNIT", "message": "..." }`

#### Scenario: Invalid currency

- **WHEN** a `PUT` request is made with `{ "currency": "DOGECOIN" }`
- **THEN** the response status SHALL be `422`
- **AND** the response body SHALL contain `{ "code": "SETTINGS_INVALID_CURRENCY", "message": "..." }`

#### Scenario: Invalid locale

- **WHEN** a `PUT` request is made with `{ "locale": "klingon" }`
- **THEN** the response status SHALL be `422`
- **AND** the response body SHALL contain `{ "code": "SETTINGS_INVALID_LOCALE", "message": "..." }`

### Requirement: Valid field domains

The API SHALL accept only values from the following domains.

#### Scenario: Valid unit_system values

- **WHEN** `unit_system` is one of `metric`, `imperial`, `custom`
- **THEN** the value SHALL be accepted

#### Scenario: Valid distance_unit values

- **WHEN** `distance_unit` is one of `km`, `mi`
- **THEN** the value SHALL be accepted

#### Scenario: Valid volume_unit values

- **WHEN** `volume_unit` is one of `l`, `gal`
- **THEN** the value SHALL be accepted

#### Scenario: Valid color_mode values

- **WHEN** `color_mode` is one of `light`, `dark`, `system`
- **THEN** the value SHALL be accepted

#### Scenario: Valid currency values

- **WHEN** `currency` is one of `USD`, `EUR`
- **THEN** the value SHALL be accepted

#### Scenario: Valid locale values

- **WHEN** `locale` is one of `en`
- **THEN** the value SHALL be accepted
