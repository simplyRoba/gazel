## ADDED Requirements

### Requirement: Health endpoint remains public
`GET /health` SHALL remain accessible without a Gazel session regardless of whether built-in authentication is enabled.

#### Scenario: Unauthenticated health check with authentication enabled
- **WHEN** built-in authentication is enabled
- **AND** an unauthenticated client sends `GET /health`
- **THEN** the health handler SHALL process the request normally
- **AND** the response SHALL NOT redirect to login or return `401 Unauthorized`
