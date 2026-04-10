## ADDED Requirements

### Requirement: Import endpoint

`POST /api/import` SHALL accept a JSON export document and write the contained vehicles and fill-ups to the database.

#### Scenario: Successful import in replace mode

- **WHEN** a `POST` request is made to `/api/import` with a valid export JSON body
- **THEN** the response status SHALL be `200`
- **AND** all existing vehicles and fill-ups SHALL be deleted
- **AND** all vehicles and fill-ups from the import document SHALL be inserted
- **AND** the response body SHALL contain a summary: `{ "vehicles_created": N, "fillups_created": M }`

#### Scenario: Successful import in merge mode

- **WHEN** a `POST` request is made to `/api/import?mode=merge` with a valid export JSON body
- **THEN** the response status SHALL be `200`
- **AND** existing data SHALL be preserved
- **AND** vehicles SHALL be matched by name (case-insensitive); matched vehicles SHALL have their fields updated and fill-ups merged
- **AND** unmatched vehicles SHALL be inserted as new
- **AND** fill-ups SHALL be matched by `(vehicle, date, odometer)` tuple; matched fill-ups SHALL be skipped, unmatched fill-ups SHALL be inserted
- **AND** the response body SHALL contain: `{ "vehicles_created": N, "vehicles_updated": U, "fillups_created": M, "fillups_skipped": S }`

#### Scenario: Default import mode is replace

- **WHEN** a `POST` request is made to `/api/import` without a `mode` query parameter
- **THEN** the import SHALL use replace mode

#### Scenario: Invalid mode parameter

- **WHEN** a `POST` request is made to `/api/import?mode=invalid`
- **THEN** the response status SHALL be `422`
- **AND** the response body SHALL contain `{ "code": "IMPORT_INVALID_MODE", "message": "..." }`

### Requirement: Import version compatibility

The import endpoint SHALL validate that the export document's version is compatible with the running server.

#### Scenario: Matching major.minor version

- **WHEN** the export document's `version` field has the same major and minor version as the running server
- **THEN** the import SHALL proceed regardless of patch version differences

#### Scenario: Mismatched major version

- **WHEN** the export document's `version` field has a different major version than the running server
- **THEN** the response status SHALL be `422`
- **AND** the response body SHALL contain `{ "code": "IMPORT_VERSION_MISMATCH", "message": "..." }`

#### Scenario: Mismatched minor version

- **WHEN** the export document's `version` field has the same major but different minor version
- **THEN** the response status SHALL be `422`
- **AND** the response body SHALL contain `{ "code": "IMPORT_VERSION_MISMATCH", "message": "..." }`

#### Scenario: Missing version field

- **WHEN** the export document does not contain a `version` field
- **THEN** the response status SHALL be `400`
- **AND** the response body SHALL contain `{ "code": "INVALID_REQUEST_BODY", "message": "..." }`
- **NOTE** The `version` field is structurally required; a missing field is a deserialization error handled by the standard JSON body extractor

### Requirement: Import validation

The import endpoint SHALL validate all records before committing any changes.

#### Scenario: Vehicle with missing required fields

- **WHEN** the import document contains a vehicle without a `name` field
- **THEN** the response status SHALL be `422`
- **AND** the response body SHALL contain `{ "code": "IMPORT_VALIDATION_ERROR", "message": "..." }`
- **AND** no data SHALL be modified

#### Scenario: Fill-up with invalid data

- **WHEN** the import document contains a fill-up with a negative odometer value
- **THEN** the response status SHALL be `422`
- **AND** the response body SHALL contain `{ "code": "IMPORT_VALIDATION_ERROR", "message": "..." }`
- **AND** no data SHALL be modified

#### Scenario: Malformed JSON body

- **WHEN** the request body is not valid JSON or does not match the export schema
- **THEN** the response status SHALL be `400`
- **AND** the response body SHALL contain `{ "code": "INVALID_REQUEST_BODY", "message": "..." }`
- **NOTE** Consistent with all other endpoints -- the standard `JsonBody` extractor handles deserialization errors

### Requirement: Import atomicity

All database changes during import SHALL occur within a single transaction.

#### Scenario: Validation failure rolls back

- **WHEN** import validation fails partway through processing
- **THEN** the transaction SHALL be rolled back
- **AND** the database SHALL remain in its pre-import state

#### Scenario: Database error during insert rolls back

- **WHEN** a database error occurs while inserting imported records
- **THEN** the transaction SHALL be rolled back
- **AND** the response status SHALL be `500`
- **AND** the response body SHALL follow the standard error shape

### Requirement: Import preview (dry run)

`POST /api/import?preview=true` SHALL validate the import document and return a summary without modifying the database.

#### Scenario: Preview of valid import in replace mode

- **WHEN** a `POST` request is made to `/api/import?preview=true` with a valid export document
- **THEN** the response status SHALL be `200`
- **AND** the response body SHALL contain `{ "preview": true, "vehicles": N, "fillups": M }`
- **AND** no data SHALL be modified

#### Scenario: Preview of valid import in merge mode

- **WHEN** a `POST` request is made to `/api/import?preview=true&mode=merge` with a valid export document
- **THEN** the response status SHALL be `200`
- **AND** the response body SHALL contain `{ "preview": true, "vehicles_new": N, "vehicles_existing": E, "fillups_new": M, "fillups_existing": S }`
- **AND** no data SHALL be modified

#### Scenario: Preview with invalid data

- **WHEN** a `POST` request is made to `/api/import?preview=true` with an invalid export document
- **THEN** the response SHALL return the same validation error as a non-preview import

### Requirement: Import body size limit

The import endpoint SHALL enforce a request body size limit.

#### Scenario: Body within limit

- **WHEN** the import request body is within 10 MB
- **THEN** the request SHALL be processed normally

#### Scenario: Body exceeds limit

- **WHEN** the import request body exceeds 10 MB
- **THEN** the response status SHALL be `413`
