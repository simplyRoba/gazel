## MODIFIED Requirements

### Requirement: Return navigation cannot become an open redirect
Gazel MUST accept only a serialized local protected UI target of at most 2,048 UTF-8 bytes after percent-decoding as `return_to`; it MUST reject absolute URLs, protocol-relative values, backslashes, control characters, oversized values, and API, health, or authentication endpoint targets. Authentication middleware can derive only the protected HTTP request path and optional query because browser URL fragments are not sent to the backend. The SPA MAY additionally place `location.hash` inside the percent-encoded `return_to` query value. Every invalid or oversized target SHALL default to `/` before redirect serialization or transaction storage.

#### Scenario: Backend navigation preserves only path and query
- **WHEN** authentication middleware handles an unauthenticated request target `/settings?tab=data`
- **THEN** it SHALL redirect to `/login?return_to=%2Fsettings%3Ftab%3Ddata`
- **AND** it SHALL NOT append, infer, or claim to have received a browser URL fragment

#### Scenario: SPA-originated hash is query-parameter data
- **WHEN** the wire request is `GET /auth/login?return_to=%2Fsettings%3Ftab%3Ddata%23export`
- **THEN** `%23export` SHALL be treated as data inside the `return_to` query parameter, not as a fragment of the `/auth/login` request
- **AND** Gazel SHALL decode and validate the serialized local target `/settings?tab=data#export`
- **AND** store that target only in the backend login transaction
- **AND** navigate to the stored target through the successful callback completion document after authentication

#### Scenario: External or reserved return target
- **WHEN** the decoded login value is external, protocol-relative, malformed, reserved, contains a backslash, or contains a control character
- **THEN** Gazel SHALL replace it with `/`
- **AND** SHALL NOT reflect the unsafe value into a `Location` header or callback completion document

#### Scenario: Return target absent
- **WHEN** login receives no `return_to`
- **THEN** the backend login transaction SHALL use `/`

#### Scenario: Return target exceeds the length limit
- **WHEN** the percent-decoded serialized return target exceeds 2,048 UTF-8 bytes
- **THEN** Gazel SHALL replace it with `/`
- **AND** authentication middleware SHALL use `/login?return_to=%2F` instead of reflecting the oversized target
- **AND** `/auth/login` SHALL store only `/` in the backend login transaction

### Requirement: Callback validation is one-time and complete
`GET /auth/callback` MUST validate and atomically consume the backend login transaction before establishing a Gazel session, so at most one concurrent callback can obtain it. After a successful callback completes all existing state, nonce, PKCE, token, issuer, audience, and session checks, rotates the session identifier, and establishes the authenticated Gazel session, it SHALL respond with `200 OK` and a minimal HTML completion document that performs a fresh same-origin navigation to the safe `return_to` stored in that transaction. The successful response SHALL use an HTML content type, SHALL prevent caching and referrer disclosure, and SHALL NOT contain a `Location` header. The completion document SHALL safely serialize the validated destination, SHALL NOT expose callback or session secrets in its content or subsequent navigation metadata, and SHALL terminate the cross-site authorization redirect chain before entering the protected SPA for WebKit/PWA compatibility. Every failed callback SHALL instead respond with `303 See Other` and redirect to `/login?error=<stable-error-code>&return_to=<encoded-safe-local-target>`. The `return_to` parameter MUST always be present; when no validated transaction target remains, Gazel SHALL use `/`, encoded as `%2F`. The authenticated session cookie SHALL retain its existing `SameSite=Lax` and CSRF properties.

#### Scenario: Valid callback
- **WHEN** the callback contains an authorization code and state matching an unexpired login transaction
- **AND** the provider accepts the matching PKCE verifier and selected token client-authentication method
- **AND** the token response contains a valid ID token whose signature, issuer, audience, expiration, and nonce all validate
- **AND** any access-token hash claim in the ID token matches the returned access token
- **THEN** Gazel SHALL rotate the session identifier
- **AND** establish an authenticated Gazel session
- **AND** update or set the authenticated session cookie with its existing protected attributes, including `SameSite=Lax`
- **AND** respond with `200 OK`, `Content-Type: text/html; charset=utf-8`, `Cache-Control: no-store`, and `Referrer-Policy: no-referrer`
- **AND** omit the `Location` header
- **AND** return a minimal HTML completion document that navigates to the validated local `return_to` using a fresh same-origin navigation
- **AND** the completion document SHALL provide a safe non-script fallback navigation when scripting is unavailable

#### Scenario: Validated return target is preserved
- **WHEN** a successful callback consumes a transaction whose validated `return_to` is `/settings?tab=data#export`
- **THEN** the `200 OK` completion document SHALL target `/settings?tab=data#export`
- **AND** it SHALL preserve the target as a same-origin navigation without exposing the original callback parameters

#### Scenario: Unsafe return target falls back to the root
- **WHEN** `/auth/login` receives a `return_to` that is absolute, protocol-relative, malformed, reserved, contains a backslash or control character, or exceeds the configured length limit
- **AND** authentication later completes successfully
- **THEN** centralized return-target validation SHALL replace the target with `/` before transaction storage
- **AND** the successful callback completion document SHALL navigate to `/`
- **AND** the unsafe value SHALL NOT be reflected into the HTML, script, or navigation response

#### Scenario: Missing or mismatched state
- **WHEN** callback state is absent, does not match the browser-bound transaction, or has no corresponding transaction
- **THEN** Gazel SHALL NOT contact the token endpoint or establish a session
- **AND** SHALL redirect to `/login?error=authentication_failed&return_to=%2F`

#### Scenario: Expired or replayed login transaction
- **WHEN** a callback uses an expired transaction or reuses a transaction already presented to a callback
- **THEN** Gazel SHALL NOT establish an authenticated session
- **AND** SHALL redirect to `/login?error=authentication_failed&return_to=%2F`

#### Scenario: Concurrent callback replay
- **WHEN** two callbacks concurrently present the same valid session-bound state
- **THEN** Gazel SHALL atomically grant the login transaction to at most one callback
- **AND** every other callback SHALL be rejected before token exchange
- **AND** every rejected callback SHALL redirect to `/login?error=authentication_failed&return_to=%2F`

#### Scenario: Invalid nonce or ID token
- **WHEN** the provider returns a missing, malformed, incorrectly signed, expired, wrong-issuer, wrong-audience, or wrong-nonce ID token
- **THEN** Gazel SHALL NOT establish an authenticated session
- **AND** SHALL redirect to `/login?error=authentication_failed&return_to=<encoded-safe-local-target>`

#### Scenario: Provider authorization or validation error
- **WHEN** the provider callback reports an error or the returned response/token fails protocol or claim validation
- **THEN** Gazel SHALL NOT expose the provider's description, callback parameters, or token details
- **AND** SHALL NOT establish a session
- **AND** SHALL redirect to `/login?error=authentication_failed&return_to=<encoded-safe-local-target>`

#### Scenario: Provider temporarily unavailable
- **WHEN** discovery-cached token or JWKS communication fails because the provider is unavailable
- **THEN** Gazel SHALL NOT establish a session
- **AND** SHALL redirect to `/login?error=provider_unavailable&return_to=<encoded-safe-local-target>`

#### Scenario: Completion document contains no sensitive authentication data
- **WHEN** a successful callback returns its completion document
- **THEN** the document SHALL contain no authorization code, access token, ID token, nonce, state, PKCE verifier, client secret, session identifier, or backend session content
- **AND** `Referrer-Policy: no-referrer` SHALL prevent the callback URL, authorization code, and state from being disclosed by script, meta-refresh, or link navigation
- **AND** the navigation destination SHALL be derived only from the centrally validated local `return_to`
