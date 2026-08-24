## MODIFIED Requirements

### Requirement: Embedded SPA serving
The server SHALL serve the SvelteKit static build embedded in the binary, falling back to `index.html` for client-side routing after any enabled authentication requirement has been satisfied.

#### Scenario: Static asset served by exact path
- **WHEN** authentication is disabled or the request has a valid session
- **AND** a request is made for a path matching an embedded static asset (e.g., `/assets/app.js`)
- **THEN** the server SHALL respond with `200 OK`
- **AND** the response SHALL include the correct `Content-Type` header inferred from the file extension

#### Scenario: SPA fallback for unknown paths
- **WHEN** authentication is disabled or the request has a valid session
- **AND** a request is made for a path that does not match any API route or static asset
- **THEN** the server SHALL respond with `200 OK` and the contents of `index.html`

#### Scenario: Unauthenticated SPA request
- **WHEN** authentication is enabled
- **AND** an unauthenticated request is made for an embedded asset or SPA fallback path
- **THEN** authentication middleware SHALL redirect the request to the public login endpoint
- **AND** embedded content SHALL NOT be served

#### Scenario: API routes take priority over SPA fallback
- **WHEN** a request is made to a path under `/api/` or to `/health`
- **THEN** the API route handler or authentication boundary SHALL process the request
- **AND** the SPA fallback SHALL NOT be invoked

## ADDED Requirements

### Requirement: Public and protected routers are composed explicitly
When authentication is enabled, the HTTP server SHALL compose exact public routes separately from one protected application router so that authentication cannot accidentally cover health or authentication endpoints or omit UI/API paths.

#### Scenario: Public route composition
- **WHEN** the enabled router is assembled
- **THEN** `/health`, `GET /auth/login`, `GET /auth/callback`, and `POST /auth/logout` SHALL bypass the authentication requirement

#### Scenario: Protected route composition
- **WHEN** the enabled router is assembled
- **THEN** `/api`, all `/api/*` routes, embedded static assets, and SPA fallback routes SHALL share one authentication boundary
