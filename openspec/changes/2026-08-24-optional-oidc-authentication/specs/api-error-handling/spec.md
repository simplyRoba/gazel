## ADDED Requirements

### Requirement: Authentication-required error maps to 401
Authentication middleware SHALL represent a missing, invalid, or expired Gazel session as `ApiError::Unauthorized("AUTHENTICATION_REQUIRED")` for API requests.

#### Scenario: Unauthorized maps to 401
- **WHEN** authentication middleware returns `ApiError::Unauthorized("AUTHENTICATION_REQUIRED")`
- **THEN** the HTTP status SHALL be `401 Unauthorized`
- **AND** the JSON body SHALL be `{ "code": "AUTHENTICATION_REQUIRED", "message": "Authentication is required." }`

### Requirement: API authentication failures never redirect
Authentication failures for `/api` and `/api/*` SHALL use the normal JSON API error shape and SHALL NOT redirect or return provider HTML.

#### Scenario: Unauthenticated API client
- **WHEN** a request without a valid Gazel session targets `/api` or any `/api/*` path
- **THEN** the response SHALL be `401 Unauthorized`
- **AND** the `Content-Type` SHALL be `application/json`
- **AND** the response SHALL contain code `AUTHENTICATION_REQUIRED`
- **AND** no `Location` header SHALL be present
