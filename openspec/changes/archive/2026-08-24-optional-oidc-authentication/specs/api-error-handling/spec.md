## ADDED Requirements

### Requirement: Authentication failures use the Unauthorized API error
Authentication middleware SHALL represent a missing, invalid, or expired Gazel session for `/api` and `/api/*` as `ApiError::Unauthorized("AUTHENTICATION_REQUIRED")`. This new variant SHALL use the existing JSON API error machinery, map to `401 Unauthorized`, and never produce a browser redirect.

#### Scenario: Unauthenticated API request maps to Unauthorized
- **WHEN** a request without a valid Gazel session targets `/api` or any `/api/*` path
- **THEN** the response SHALL be `401 Unauthorized`
- **AND** the JSON error code SHALL be `AUTHENTICATION_REQUIRED`
- **AND** the fallback message SHALL be `Authentication is required.`
- **AND** no `Location` header SHALL be present
