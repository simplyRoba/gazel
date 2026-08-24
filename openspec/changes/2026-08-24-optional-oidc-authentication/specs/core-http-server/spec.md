## MODIFIED Requirements

### Requirement: Embedded SPA serving
The server SHALL serve the embedded SvelteKit build with exact shared static assets available to the public login page, a public `/login` document, and authenticated SPA fallback for application routes when built-in authentication is enabled.

#### Scenario: Static asset served by exact path
- **WHEN** a request exactly matches a non-HTML embedded asset required by the login or application UI (e.g., `/_app/immutable/...`, a logo, manifest, or favicon)
- **THEN** the server SHALL respond with `200 OK` without requiring a Gazel session
- **AND** the response SHALL include the correct `Content-Type` header inferred from the file extension
- **AND** the asset SHALL contain no application data or OIDC token
- **AND** `index.html` SHALL NOT be included in the public exact-asset allowlist

#### Scenario: SPA fallback for unknown paths
- **WHEN** authentication is disabled or the request has a valid session
- **AND** a request is made for a path that does not match any API route or static asset
- **THEN** the server SHALL respond with `200 OK` and the contents of `index.html`

#### Scenario: Public login document
- **WHEN** authentication is enabled and a browser requests `GET /login`
- **THEN** the server SHALL respond with `index.html` without requiring a Gazel session
- **AND** SvelteKit SHALL render the dedicated login route

#### Scenario: Index and nonexistent paths are protected
- **WHEN** authentication is enabled and an unauthenticated client requests `/index.html` or a nonexistent asset path
- **THEN** the public asset handler SHALL NOT return `index.html`
- **AND** the protected application boundary SHALL process the request
- **AND** browser navigation SHALL receive the normal `/login` redirect

#### Scenario: Unauthenticated application navigation
- **WHEN** authentication is enabled
- **AND** an unauthenticated browser navigation requests any application SPA path other than `/login`
- **THEN** authentication middleware SHALL redirect to `/login` with the local request target encoded as `return_to`
- **AND** embedded application content SHALL NOT be served

#### Scenario: API routes take priority over SPA fallback
- **WHEN** a request is made to a path under `/api/` or to `/health`
- **THEN** the API route handler or authentication boundary SHALL process the request
- **AND** the SPA fallback SHALL NOT be invoked

## ADDED Requirements

### Requirement: Public and protected routers are composed explicitly
Existing disabled application/API routes SHALL retain their current behavior while exposing only an inert public auth-config status for the compiled login route. When authentication is enabled, exact public resources SHALL be composed separately from one protected application router.

#### Scenario: Disabled route compatibility
- **WHEN** authentication is disabled
- **THEN** existing application, API, health, static, and fallback behavior SHALL remain unchanged
- **AND** `GET /auth/config` SHALL return only `{ "enabled": false }`

#### Scenario: Public route composition
- **WHEN** the enabled router is assembled
- **THEN** `/health`, `GET /login`, `GET /auth/config`, `GET /auth/login`, `GET /auth/callback`, `POST /auth/logout`, and exact non-HTML embedded assets SHALL bypass the authentication requirement

#### Scenario: Protected route composition
- **WHEN** the enabled router is assembled
- **THEN** `/api`, all `/api/*` routes, direct `/index.html`, and every application SPA document/fallback other than `/login` SHALL share one authentication boundary
