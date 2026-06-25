## ADDED Requirements

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

### Requirement: Access log middleware

The server SHALL log every HTTP request at `debug` level with method, path, response status, and elapsed time.

#### Scenario: Request is logged
- **WHEN** any HTTP request is processed
- **THEN** a `debug`-level log entry SHALL be emitted containing the HTTP method, request path, response status code, and duration in milliseconds

### Requirement: Embedded SPA serving

The server SHALL serve the SvelteKit static build embedded in the binary, falling back to `index.html` for client-side routing.

#### Scenario: Static asset served by exact path
- **WHEN** a request is made for a path matching an embedded static asset (e.g., `/assets/app.js`)
- **THEN** the server SHALL respond with `200 OK`
- **AND** the response SHALL include the correct `Content-Type` header inferred from the file extension

#### Scenario: SPA fallback for unknown paths
- **WHEN** a request is made for a path that does not match any API route or static asset
- **THEN** the server SHALL respond with `200 OK` and the contents of `index.html`

#### Scenario: API routes take priority over SPA fallback
- **WHEN** a request is made to a path under `/api/` or to `/health`
- **THEN** the API route handler SHALL process the request
- **AND** the SPA fallback SHALL NOT be invoked

### Requirement: API route namespace

All domain API endpoints SHALL be nested under the `/api` path prefix. The `/health` endpoint SHALL be at the root level, outside `/api`.

#### Scenario: API namespace structure
- **WHEN** the router is assembled
- **THEN** domain endpoints SHALL be accessible under `/api/*`
- **AND** the health endpoint SHALL be accessible at `/health`

### Requirement: Application state with pool extraction

The application SHALL use an `AppState` struct that implements `FromRef` for `SqlitePool`, allowing handlers to extract the database pool directly.

#### Scenario: Handler extracts pool from state
- **WHEN** a handler declares `State<SqlitePool>` as a parameter
- **THEN** the pool SHALL be extracted from `AppState` via the `FromRef` implementation
