## Purpose

Defines Gazel’s public branded Svelte login page, its single provider-labelled OIDC action, safe return propagation, and usable failure and signed-out states.

## Requirements

### Requirement: Public login page presents one OIDC action
When built-in authentication is enabled, Gazel SHALL serve a public Svelte page at `/login` without requiring an authenticated session.

#### Scenario: Default login page
- **WHEN** an unauthenticated browser navigates to `/login`
- **THEN** the page SHALL display Gazel branding
- **AND** a short translated message that authentication is required to use Gazel
- **AND** exactly one OIDC login button
- **AND** no username, email, password, registration, or other local-user control

#### Scenario: Valid session bypasses login page
- **WHEN** a browser with a valid Gazel session requests `/login`
- **THEN** the backend SHALL redirect to `/` before serving the Svelte login page
- **AND** the browser SHALL NOT display the authentication-required state

#### Scenario: Provider-labelled button
- **WHEN** the public auth config reports provider name `Authentik`
- **THEN** the button SHALL render the translated label `Continue with Authentik`

#### Scenario: Default provider label
- **WHEN** no provider name is configured
- **THEN** the public auth config SHALL report `OpenID Connect`
- **AND** the button SHALL render the translated label `Continue with OpenID Connect`

### Requirement: Login page starts OIDC and preserves local return target
The login button SHALL navigate to `/auth/login` and propagate only the current decoded `return_to` query value for backend validation.

#### Scenario: SPA-originated encoded settings return target
- **WHEN** SPA expiry handling navigates to `/login?return_to=%2Fsettings%3Ftab%3Ddata%23export`
- **THEN** the OIDC button target SHALL be `/auth/login?return_to=%2Fsettings%3Ftab%3Ddata%23export`
- **AND** `%23export` SHALL remain query-parameter data rather than becoming a fragment of the `/auth/login` request
- **AND** successful authentication SHALL ultimately navigate to `/settings?tab=data#export`

#### Scenario: Return target absent
- **WHEN** `/login` has no `return_to`
- **THEN** the OIDC button SHALL start `/auth/login` with `/` as the effective return target

#### Scenario: Unsafe return target
- **WHEN** `/login` receives an absolute, protocol-relative, malformed, reserved, or over-2,048-byte decoded `return_to`
- **THEN** frontend handling SHALL NOT convert it into an external navigation
- **AND** `/auth/login` SHALL remain authoritative and default it to `/`

### Requirement: Login page exposes safe failure and signed-out states
The page SHALL render stable local status codes without displaying provider descriptions, callback parameters, or secrets.

#### Scenario: Authentication failed
- **WHEN** the page URL contains `error=authentication_failed` and an encoded `return_to`
- **THEN** the page SHALL display a translated authentication-failed alert
- **AND** retain the one OIDC button so the user can retry
- **AND** the retry button SHALL propagate that `return_to` through the normal validation flow

#### Scenario: Provider temporarily unavailable
- **WHEN** the page URL contains `error=provider_unavailable`
- **THEN** the page SHALL display a translated temporary-unavailability alert
- **AND** retain the one OIDC button so the user can retry

#### Scenario: Signed out
- **WHEN** the page URL contains `logged_out=1`
- **THEN** the page SHALL display a translated confirmation that the local Gazel session ended
- **AND** SHALL NOT automatically initiate OIDC
- **AND** retain the one explicit OIDC button

#### Scenario: Unknown error value
- **WHEN** the page receives an unrecognized `error` value
- **THEN** it SHALL display the generic authentication-failed alert
- **AND** SHALL NOT reflect the unknown value as HTML

### Requirement: Login page depends only on public resources
The public login page SHALL load only the shared static assets and public auth-config endpoint needed to render and start OIDC.

#### Scenario: Login page initialization
- **WHEN** `/login` mounts without a Gazel session
- **THEN** it SHALL request `GET /auth/config` for the provider display name
- **AND** SHALL NOT call `/api/settings`, `/api/vehicles`, or any other protected application API

#### Scenario: Authentication disabled
- **WHEN** public auth config reports `enabled: false`
- **THEN** the login route SHALL replace browser navigation with `/`
- **AND** SHALL NOT display the authentication-required text or OIDC button

#### Scenario: Auth config unavailable
- **WHEN** the public auth-config request fails or returns malformed data
- **THEN** the page SHALL show a usable generic unavailable state
- **AND** SHALL NOT expose a broken username/password fallback
