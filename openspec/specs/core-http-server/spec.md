## Purpose

HTTP server behavior: configured port binding, graceful shutdown, access logging, embedded browser application serving, the API route namespace, and authentication access boundaries.

## Requirements

### Requirement: Server binds to configured port

The server SHALL listen for HTTP connections on the port specified by the `GAZEL_PORT` configuration value.

#### Scenario: Default port binding
- **WHEN** the application starts without `GAZEL_PORT` set
- **THEN** the server SHALL bind to port `4110`

#### Scenario: Custom port binding
- **WHEN** the application starts with `GAZEL_PORT=8080`
- **THEN** the server SHALL bind to port `8080`

### Requirement: Graceful shutdown on termination signals

The server SHALL shut down gracefully when receiving a termination signal, allowing in-flight requests to complete before exiting.

#### Scenario: Shutdown on SIGINT
- **WHEN** the process receives `SIGINT` (Ctrl+C)
- **THEN** the server SHALL stop accepting new connections
- **AND** the server SHALL wait for in-flight requests to complete
- **AND** the process SHALL exit with code 0

#### Scenario: Shutdown on SIGTERM
- **WHEN** the process receives `SIGTERM` (Docker stop)
- **THEN** the server SHALL stop accepting new connections
- **AND** the server SHALL wait for in-flight requests to complete
- **AND** the process SHALL exit with code 0

### Requirement: HTTP access logging

The server SHALL log every HTTP request at `debug` level with method, path, response status, and elapsed time.

#### Scenario: Request is logged
- **WHEN** any HTTP request is processed
- **THEN** a `debug`-level log entry SHALL be emitted containing the HTTP method, request path, response status code, and duration in milliseconds

### Requirement: Embedded browser application serving
The server SHALL serve the embedded browser application with exact shared static assets available to the public login page, a public `/login` document, and authenticated fallback for application routes when built-in authentication is enabled.

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
- **WHEN** authentication is enabled and a browser without a valid Gazel session requests `GET /login`
- **THEN** the server SHALL respond with `index.html` without requiring a Gazel session
- **AND** the browser application SHALL render the dedicated login route

#### Scenario: Authenticated login navigation
- **WHEN** authentication is enabled and a browser with a valid Gazel session requests `GET /login`
- **THEN** the server SHALL redirect with `303 See Other` to `/`
- **AND** SHALL NOT serve `index.html` for the login route

#### Scenario: Index and nonexistent paths are protected
- **WHEN** authentication is enabled and an unauthenticated client requests `/index.html` or a nonexistent asset path
- **THEN** the request SHALL NOT receive `index.html` as a public asset
- **AND** browser navigation SHALL receive the normal `/login` redirect

#### Scenario: Unauthenticated application navigation
- **WHEN** authentication is enabled
- **AND** an unauthenticated browser navigation requests any application SPA path other than `/login`
- **THEN** the server SHALL redirect to `/login` with only the HTTP request path and optional query encoded as `return_to`
- **AND** it SHALL NOT infer or claim to preserve a browser URL fragment that was not sent to the server
- **AND** embedded application content SHALL NOT be served

#### Scenario: API routes take priority over SPA fallback
- **WHEN** a request is made to `/api`, a path under `/api/`, or `/health`
- **THEN** the request SHALL receive the applicable API, health, or authentication response
- **AND** the browser-application fallback SHALL NOT be served

### Requirement: API route namespace

All domain API endpoints SHALL be nested under the `/api` path prefix. The `/health` endpoint SHALL be at the root level, outside `/api`.

#### Scenario: API namespace structure
- **WHEN** the application serves HTTP requests
- **THEN** domain endpoints SHALL be accessible under `/api/*`
- **AND** the health endpoint SHALL be accessible at `/health`

### Requirement: Public and protected access boundaries
When authentication is disabled, existing application, API, health, static, and fallback behavior SHALL remain unchanged except for the inert public `GET /auth/config` endpoint. When authentication is enabled, only explicitly public resources SHALL be accessible without a valid Gazel session; API and application resources SHALL remain protected.

#### Scenario: Disabled route compatibility
- **WHEN** authentication is disabled
- **THEN** existing application, API, health, static, and fallback behavior SHALL remain unchanged
- **AND** `GET /auth/config` SHALL return only `{ "enabled": false }`

#### Scenario: Public resources
- **WHEN** authentication is enabled
- **THEN** `/health`, `GET /login`, `GET /auth/config`, `GET /auth/login`, `GET /auth/callback`, `POST /auth/logout`, and exact non-HTML embedded assets SHALL be accessible without a valid Gazel session

#### Scenario: Protected resources
- **WHEN** authentication is enabled
- **THEN** `/api`, all `/api/*` routes, direct `/index.html`, and every application document or fallback other than `/login` SHALL require a valid Gazel session
