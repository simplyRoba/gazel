## ADDED Requirements

### Requirement: Consistent JSON error responses

All API error responses SHALL return a JSON body with `code` and `message` fields. The `code` SHALL be a static string identifier and the `message` SHALL be a human-readable description.

#### Scenario: Error response format
- **WHEN** any API endpoint returns an error
- **THEN** the response body SHALL be JSON with the shape `{ "code": "<ERROR_CODE>", "message": "<human-readable message>" }`
- **AND** the `Content-Type` header SHALL be `application/json`

### Requirement: Error variant to HTTP status mapping

The `ApiError` enum SHALL map each variant to the appropriate HTTP status code.

#### Scenario: NotFound maps to 404
- **WHEN** a handler returns `ApiError::NotFound`
- **THEN** the HTTP status SHALL be `404 Not Found`

#### Scenario: Validation maps to 422
- **WHEN** a handler returns `ApiError::Validation`
- **THEN** the HTTP status SHALL be `422 Unprocessable Entity`

#### Scenario: Conflict maps to 409
- **WHEN** a handler returns `ApiError::Conflict`
- **THEN** the HTTP status SHALL be `409 Conflict`

#### Scenario: BadRequest maps to 400
- **WHEN** a handler returns `ApiError::BadRequest`
- **THEN** the HTTP status SHALL be `400 Bad Request`

#### Scenario: InternalError maps to 500
- **WHEN** a handler returns `ApiError::InternalError`
- **THEN** the HTTP status SHALL be `500 Internal Server Error`

### Requirement: Database error helper

A `db_error` helper function SHALL convert `sqlx::Error` into `ApiError::InternalError`, logging the underlying error without exposing database details to the client.

#### Scenario: Database error is wrapped
- **WHEN** a database query fails with a `sqlx::Error`
- **THEN** `db_error()` SHALL log the error at `error` level
- **AND** SHALL return `ApiError::InternalError("INTERNAL_ERROR")`
- **AND** the original database error details SHALL NOT appear in the HTTP response

### Requirement: Custom JSON body extractor

A `JsonBody<T>` extractor SHALL wrap `axum::Json<T>` and return `ApiError::BadRequest("INVALID_REQUEST_BODY")` when the request body cannot be parsed, instead of axum's default plain-text error.

#### Scenario: Valid JSON body
- **WHEN** a request with a valid JSON body is received by a handler using `JsonBody<T>`
- **THEN** the body SHALL be deserialized into `T` and the handler SHALL proceed normally

#### Scenario: Malformed JSON body
- **WHEN** a request with malformed JSON is received by a handler using `JsonBody<T>`
- **THEN** the response SHALL be `400 Bad Request`
- **AND** the body SHALL be `{ "code": "INVALID_REQUEST_BODY", "message": "..." }`

#### Scenario: Missing required fields
- **WHEN** a request body is valid JSON but missing required fields for `T`
- **THEN** the response SHALL be `400 Bad Request`
- **AND** the body SHALL be `{ "code": "INVALID_REQUEST_BODY", "message": "..." }`

### Requirement: Nullable field deserialization helper

The error module SHALL provide a `deserialize_nullable` function for use with serde's `deserialize_with` attribute, enabling three-state PATCH semantics (`Option<Option<T>>`).

#### Scenario: Field absent from JSON
- **WHEN** a JSON request body omits a field annotated with `deserialize_nullable`
- **THEN** the field SHALL deserialize as `None` (outer Option)

#### Scenario: Field explicitly set to null
- **WHEN** a JSON request body sets a field to `null`
- **THEN** the field SHALL deserialize as `Some(None)` (inner Option is None)

#### Scenario: Field set to a value
- **WHEN** a JSON request body sets a field to a non-null value
- **THEN** the field SHALL deserialize as `Some(Some(value))`

### Requirement: Vehicle error codes

The `default_message` function SHALL map vehicle-specific error codes to human-readable messages.

#### Scenario: Vehicle error code messages
- **WHEN** an error response uses a vehicle error code
- **THEN** the following mappings SHALL apply:
- **AND** `VEHICLE_NOT_FOUND` SHALL map to `"Vehicle not found."`
- **AND** `VEHICLE_NAME_REQUIRED` SHALL map to `"Vehicle name is required."`
- **AND** `VEHICLE_INVALID_FUEL_TYPE` SHALL map to `"Invalid fuel type."`
- **AND** `VEHICLE_INVALID_YEAR` SHALL map to `"Invalid year."`
- **AND** `VEHICLE_HAS_FILLUPS` SHALL map to `"Cannot delete vehicle with existing fill-ups."`
