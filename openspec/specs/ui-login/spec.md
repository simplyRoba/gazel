## Purpose

Defines Gazel’s public branded login page, its single provider-labelled OIDC action, safe return propagation, and usable failure and signed-out states.

## Requirements

### Requirement: Public login page presents one OIDC action
When built-in authentication is enabled, Gazel SHALL serve a public page at `/login` without requiring an authenticated session.

#### Scenario: Default login page
- **WHEN** an unauthenticated browser navigates to `/login`
- **THEN** the page SHALL display Gazel branding
- **AND** a short translated message that authentication is required to use Gazel
- **AND** exactly one OIDC login button
- **AND** no username, email, password, registration, or other local-user control

#### Scenario: Valid session bypasses login page
- **WHEN** a browser with a valid Gazel session requests `/login`
- **THEN** the backend SHALL redirect to `/` before serving the login page
- **AND** the browser SHALL NOT display the authentication-required state

#### Scenario: Provider-labelled button
- **WHEN** the public auth config reports provider name `Authentik`
- **THEN** the button SHALL render the translated label `Continue with Authentik`

#### Scenario: Default provider label
- **WHEN** no provider name is configured
- **THEN** the public auth config SHALL report `OpenID Connect`
- **AND** the button SHALL render the translated label `Continue with OpenID Connect`

### Requirement: Login page starts OIDC and preserves local return target
The login button SHALL navigate to `/auth/login` with a validated serialized local return target. Query extraction SHALL form-decode once; frontend validation SHALL apply the same acceptance vectors as backend return navigation, measure the 2,048 UTF-8 byte limit before validation-only percent decoding, and substitute `/` for invalid values. Accepted targets SHALL retain their original serialization and SHALL be encoded as one query value, not normalized or recursively decoded.

#### Scenario: SPA-originated encoded settings return target
- **WHEN** SPA expiry handling navigates to `/login?return_to=%2Fsettings%3Ftab%3Ddata%23export`
- **THEN** the OIDC button target SHALL be `/auth/login?return_to=%2Fsettings%3Ftab%3Ddata%23export`
- **AND** `%23export` SHALL remain query-parameter data rather than becoming a fragment of the `/auth/login` request
- **AND** successful authentication SHALL ultimately navigate to `/settings?tab=data#export`

#### Scenario: Return target absent
- **WHEN** `/login` has no `return_to`
- **THEN** the OIDC button SHALL start `/auth/login` with `/` as the effective return target

#### Scenario: Unsafe return target
- **WHEN** `/login` receives an absolute, protocol-relative, malformed, reserved, or over-2,048-byte serialized `return_to`
- **THEN** frontend handling SHALL substitute `/` before constructing the provider action
- **AND** SHALL NOT convert it into an external navigation
- **AND** `/auth/login` SHALL remain authoritative and independently default invalid targets to `/`

#### Scenario: Frontend and backend reject the same ambiguous targets
- **WHEN** a serialized target has other than exactly one leading slash, malformed percent escapes or UTF-8, raw or decoded backslashes or Unicode controls U+0000–U+001F or U+007F–U+009F, dot path segments, encoded path separators `/`, `\`, `?`, or `#`, or residual percent or nested escapes in its path
- **THEN** the login page SHALL use `/` before constructing the provider action, matching backend validation

#### Scenario: Reserved prefix vectors match backend policy
- **WHEN** the return path names `/auth`, `/login`, `/api`, or `/health`, or a descendant or encoded equivalent of those paths
- **THEN** the login page SHALL use `/`
- **AND** an unrelated path such as `/apiary` SHALL NOT be rejected merely for sharing a prefix
- **AND** SHALL NOT reject `/uploads` solely for that prefix in Gazel

#### Scenario: Serialized byte limit rather than wire or decoded length
- **WHEN** one form-decoding of the query value produces a serialized target exceeding 2,048 UTF-8 bytes even though validation decoding would shorten it
- **THEN** the login page SHALL use `/`
- **AND** wire query encoding size alone SHALL NOT disqualify a serialized target within the limit

#### Scenario: Valid escaped query and fragment remain intact
- **WHEN** a query value form-decodes once to `/settings?label=100%25&next=a%2Fb#part%20one`
- **THEN** the login button SHALL carry that exact serialized target encoded as one `return_to` value
- **AND** SHALL NOT reject a valid query escape because validation decoding produces a literal percent

#### Scenario: Login remains an explicit user action by default
- **WHEN** the public login page finishes loading with valid enabled configuration and `auto_redirect: false`, including a failure or retry state
- **THEN** it SHALL wait for the user to activate the provider button rather than automatically navigate to provider authorization

### Requirement: Optional automatic login uses the validated provider action
On ordinary `/login` entry with valid enabled public configuration and `auto_redirect: true`, the mounted page SHALL perform exactly one top-level navigation to `/auth/login`, replacing the login history entry and using the same validated, serialized `return_to` as the manual button. This SHALL apply to direct browser entry and SPA/API-expiry entry. The page SHALL NOT navigate automatically when the query contains `error` or `logged_out`, regardless of value, while configuration is loading, failed, or malformed, or after unmount.

#### Scenario: Ordinary automatic-login entry
- **WHEN** a direct browser entry or SPA/API-expiry navigation reaches `/login` without `error` or `logged_out` and valid enabled config reports `auto_redirect: true`
- **THEN** the page SHALL replace the login history entry with exactly one top-level navigation to the manual button's `/auth/login` target
- **AND** SHALL preserve the same validated `return_to`, including `/` for absent or unsafe targets

#### Scenario: Result query suppresses automatic login
- **WHEN** `/login` contains `error` or `logged_out`, including an empty or unknown value, even with `auto_redirect: true`
- **THEN** the page SHALL NOT automatically start OIDC
- **AND** existing generic result messages and the manual retry action SHALL remain unchanged

#### Scenario: Configuration cannot trigger navigation after unmount
- **WHEN** config is pending, fails, is malformed, or completes after the login page unmounts
- **THEN** the page SHALL NOT automatically navigate to `/auth/login`

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
The public login page SHALL load only the shared static assets and public auth-config endpoint needed to render and start OIDC. Its strict parser SHALL accept only exactly `{ "enabled": true, "provider_name": <string>, "auto_redirect": <boolean> }` or exactly `{ "enabled": false }`.

#### Scenario: Login page initialization
- **WHEN** `/login` mounts without a Gazel session
- **THEN** it SHALL request `GET /auth/config` for the provider display name and automatic-login flag
- **AND** SHALL NOT call `/api/settings`, `/api/vehicles`, or any other protected application API

#### Scenario: Auth config is loading
- **WHEN** the public auth-config request is pending
- **THEN** the page SHALL render the branded login card with a translated loading state
- **AND** SHALL NOT render or enable an OIDC provider action

#### Scenario: Authentication disabled
- **WHEN** public auth config reports `enabled: false`
- **THEN** the login route SHALL replace browser navigation with `/`
- **AND** SHALL NOT display the authentication-required text or OIDC button

#### Scenario: Invalid auth config shape
- **WHEN** public auth config has missing or non-boolean `enabled`, missing or non-string enabled `provider_name`, missing or non-boolean enabled `auto_redirect`, unexpected fields, or any disabled-mode field other than `enabled`
- **THEN** the page SHALL reject it as malformed, including the old enabled shape without `auto_redirect`
- **AND** SHALL NOT render or enable an OIDC provider action

#### Scenario: Auth config unavailable
- **WHEN** the public auth-config request fails or returns malformed data
- **THEN** the page SHALL show a usable generic unavailable state
- **AND** SHALL NOT expose a broken username/password fallback
