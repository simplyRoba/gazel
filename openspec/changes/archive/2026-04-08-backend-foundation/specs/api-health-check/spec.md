## ADDED Requirements

### Requirement: Health endpoint reports status and version

The `GET /health` endpoint SHALL verify database connectivity and return the application status and version.

#### Scenario: Healthy system
- **WHEN** a `GET /health` request is received
- **AND** the database is reachable
- **THEN** the response status SHALL be `200 OK`
- **AND** the response body SHALL be JSON with `{ "status": "ok", "version": "<cargo package version>" }`

#### Scenario: Database unreachable
- **WHEN** a `GET /health` request is received
- **AND** the database query fails
- **THEN** the response status SHALL be `503 Service Unavailable`
- **AND** the response body SHALL be JSON with `{ "status": "unhealthy" }`

### Requirement: Health endpoint is outside API namespace

The health endpoint SHALL be mounted at `/health` (root level), not under `/api/health`.

#### Scenario: Health endpoint path
- **WHEN** a `GET /health` request is received
- **THEN** the health handler SHALL process the request
- **AND** `GET /api/health` SHALL NOT route to the health handler
