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
