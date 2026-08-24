## MODIFIED Requirements

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

#### Scenario: Unauthorized maps to 401
- **WHEN** authentication middleware returns `ApiError::Unauthorized("AUTHENTICATION_REQUIRED")`
- **THEN** the HTTP status SHALL be `401 Unauthorized`
- **AND** the JSON body SHALL be `{ "code": "AUTHENTICATION_REQUIRED", "message": "Authentication is required." }`

#### Scenario: InternalError maps to 500
- **WHEN** a handler returns `ApiError::InternalError`
- **THEN** the HTTP status SHALL be `500 Internal Server Error`

## ADDED Requirements

### Requirement: API authentication failures never redirect
Authentication failures for `/api` and `/api/*` SHALL use the normal JSON API error shape and SHALL NOT redirect or return provider HTML.

#### Scenario: Unauthenticated API client
- **WHEN** a request without a valid Gazel session targets `/api` or any `/api/*` path
- **THEN** the response SHALL be `401 Unauthorized`
- **AND** the `Content-Type` SHALL be `application/json`
- **AND** the response SHALL contain code `AUTHENTICATION_REQUIRED`
- **AND** no `Location` header SHALL be present
