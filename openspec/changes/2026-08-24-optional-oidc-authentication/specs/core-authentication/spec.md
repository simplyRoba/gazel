## Purpose

Defines Gazel’s optional generic OIDC authentication boundary, secure authorization-code exchange, backend session lifecycle, and identity-neutral access policy.

## ADDED Requirements

### Requirement: Authentication is optional and disabled by default
Gazel SHALL enforce no authentication when built-in authentication is disabled, and all existing UI and API behavior SHALL remain unchanged.

#### Scenario: Default startup has no authentication gate
- **WHEN** Gazel starts without authentication enabled
- **THEN** UI navigation SHALL be served without a login
- **AND** `/api/*` requests SHALL be processed exactly as before this capability was added
- **AND** OIDC discovery SHALL NOT be attempted

### Requirement: Enabled authentication protects the application
When built-in authentication is enabled, Gazel MUST require a valid Gazel session for the embedded UI, static application assets, and every `/api` and `/api/*` route.

#### Scenario: Unauthenticated UI navigation
- **WHEN** an unauthenticated browser requests a protected non-API route
- **THEN** Gazel SHALL redirect the browser to `GET /auth/login`
- **AND** it SHALL NOT serve embedded UI content

#### Scenario: Unauthenticated API request
- **WHEN** an unauthenticated client requests `/api` or any `/api/*` path
- **THEN** Gazel SHALL return `401 Unauthorized`
- **AND** it SHALL NOT redirect to an OIDC provider or return HTML

#### Scenario: Authenticated application request
- **WHEN** a request carries a valid, unexpired Gazel session
- **THEN** the requested UI, static asset, or API route SHALL be processed normally

#### Scenario: Public routes remain reachable
- **WHEN** authentication is enabled and a client requests `/health`, `GET /auth/login`, `GET /auth/callback`, or `POST /auth/logout`
- **THEN** the route SHALL be processed without requiring an authenticated Gazel session

### Requirement: OIDC provider discovery is fail-closed
When authentication is enabled, Gazel MUST perform OpenID Connect Discovery from the configured issuer during startup and MUST validate the discovered issuer and required Authorization Code flow endpoints and signing metadata.

#### Scenario: Valid provider discovery
- **WHEN** the configured issuer returns valid standards-compliant discovery metadata
- **AND** discovery retrieves an associated JSON Web Key Set containing at least one signing-verification key compatible with an advertised ID-token signing algorithm
- **THEN** Gazel SHALL initialize its OIDC client
- **AND** startup SHALL continue

#### Scenario: Unreachable or malformed provider discovery
- **WHEN** discovery or associated JWKS retrieval fails, redirects, returns malformed data, returns a different issuer, omits a required endpoint, returns an empty JWKS, or supplies unusable signing metadata
- **THEN** Gazel startup SHALL fail
- **AND** Gazel SHALL NOT fall back to unauthenticated operation

### Requirement: Login uses Authorization Code flow with state, nonce, and PKCE
`GET /auth/login` MUST initiate generic OIDC Authorization Code flow using a fresh state value, fresh nonce, and a SHA-256 PKCE challenge for each attempt.

#### Scenario: Login initiation
- **WHEN** an unauthenticated client requests `GET /auth/login`
- **THEN** Gazel SHALL create a short-lived backend login transaction bound to that client’s session
- **AND** the transaction SHALL contain the state, nonce, and PKCE verifier
- **AND** Gazel SHALL redirect to the discovered authorization endpoint with `response_type=code`, the `openid` scope, the configured client ID, the configured external callback URL, the state, nonce, PKCE challenge, and `code_challenge_method=S256`

#### Scenario: Fresh login security values
- **WHEN** the same unauthenticated client initiates two login attempts
- **THEN** each attempt SHALL replace the prior login transaction
- **AND** each authorization request SHALL use newly generated state, nonce, and PKCE values

#### Scenario: Login requested with a valid session
- **WHEN** an already authenticated client requests `GET /auth/login`
- **THEN** Gazel SHALL redirect to `/`
- **AND** SHALL NOT invalidate or replace the authenticated session
- **AND** SHALL NOT initiate a provider authorization request

### Requirement: Callback validation is one-time and complete
`GET /auth/callback` MUST validate and atomically consume the backend login transaction before establishing a Gazel session, so at most one concurrent callback can obtain it.

#### Scenario: Valid callback
- **WHEN** the callback contains an authorization code and state matching an unexpired login transaction
- **AND** the provider accepts the matching PKCE verifier
- **AND** the token response contains a valid ID token whose signature, issuer, audience, expiration, and nonce all validate
- **AND** any access-token hash claim in the ID token matches the returned access token
- **THEN** Gazel SHALL rotate the session identifier
- **AND** establish an authenticated Gazel session
- **AND** redirect the browser to `/`

#### Scenario: Missing or mismatched state
- **WHEN** the callback state is absent, does not match the login transaction, or has no corresponding transaction
- **THEN** Gazel SHALL reject the callback
- **AND** SHALL NOT contact the token endpoint
- **AND** SHALL NOT establish an authenticated session

#### Scenario: Expired or replayed login transaction
- **WHEN** a callback uses an expired transaction or reuses a transaction already presented to a callback
- **THEN** Gazel SHALL reject the callback
- **AND** SHALL NOT establish an authenticated session

#### Scenario: Concurrent callback replay
- **WHEN** two callbacks concurrently present the same valid session-bound state
- **THEN** Gazel SHALL atomically grant the login transaction to at most one callback
- **AND** every other callback SHALL be rejected before token exchange

#### Scenario: Invalid nonce or ID token
- **WHEN** the provider returns a missing, malformed, incorrectly signed, expired, wrong-issuer, wrong-audience, or wrong-nonce ID token
- **THEN** Gazel SHALL reject the callback
- **AND** SHALL NOT establish an authenticated session

#### Scenario: Provider authorization error
- **WHEN** the provider callback contains an OAuth/OIDC error instead of an authorization code
- **THEN** Gazel SHALL reject the callback with a generic browser-safe error
- **AND** SHALL NOT expose the provider’s error description or establish a session

#### Scenario: Malformed token response
- **WHEN** the token endpoint fails or returns a malformed or incomplete response
- **THEN** Gazel SHALL reject the callback with a generic browser-safe error
- **AND** SHALL NOT establish an authenticated session

### Requirement: Sessions are backend-managed and confidential
Gazel MUST store login-transaction and authenticated-session data only on the backend. The browser MUST receive only an opaque session identifier protected by authenticated encryption in an HTTP-only cookie, and OIDC tokens MUST NOT be stored in browser storage or returned to frontend code.

#### Scenario: Session cookie attributes over HTTPS
- **WHEN** a login transaction or authenticated session is issued for an HTTPS external URL
- **THEN** the cookie SHALL be encrypted and integrity-protected
- **AND** SHALL have `HttpOnly`, `Secure`, `SameSite=Lax`, and `Path=/` attributes
- **AND** SHALL NOT contain an OIDC access token, refresh token, or ID token

#### Scenario: Loopback HTTP development
- **WHEN** the configured external URL is HTTP on a loopback host
- **THEN** the session cookie SHALL retain authenticated encryption, `HttpOnly`, `SameSite=Lax`, and `Path=/`
- **AND** the `Secure` attribute SHALL be disabled only so the loopback callback can function

#### Scenario: Invalid session identifier
- **WHEN** a request carries a malformed, tampered, or unknown session cookie
- **THEN** Gazel SHALL treat the request as unauthenticated

#### Scenario: Authenticated session expiry
- **WHEN** the absolute `expires_at` recorded at authentication reaches twelve hours after successful callback
- **THEN** Gazel SHALL invalidate the session without extending it on ordinary requests
- **AND** subsequent protected requests SHALL be treated as unauthenticated

#### Scenario: OIDC tokens stay backend-side
- **WHEN** a valid token response has been verified
- **THEN** Gazel SHALL retain only the minimal local authenticated-session marker and expiry
- **AND** SHALL discard provider tokens rather than expose or persist them

### Requirement: Local logout always invalidates the Gazel session
`POST /auth/logout` MUST destroy the backend Gazel session and expire its browser cookie without requiring provider logout support.

#### Scenario: Authenticated logout
- **WHEN** a client posts to `/auth/logout` with a valid Gazel session
- **THEN** Gazel SHALL delete the server-managed session
- **AND** expire the session cookie
- **AND** return `204 No Content`
- **AND** subsequent use of the former cookie SHALL be unauthenticated

#### Scenario: Logout without a valid session
- **WHEN** a client posts to `/auth/logout` without a valid Gazel session
- **THEN** Gazel SHALL still return `204 No Content`
- **AND** provide an expired session cookie when applicable

### Requirement: Authentication grants uniform shared access
Gazel SHALL accept any identity whose OIDC authentication validates and SHALL NOT implement local accounts or claim-based authorization.

#### Scenario: Successfully authenticated identity
- **WHEN** any provider identity completes a valid OIDC flow
- **THEN** that identity SHALL receive the same access to Gazel as every other authenticated identity
- **AND** application data SHALL remain shared rather than separated per identity

#### Scenario: No local identity management
- **WHEN** built-in authentication is enabled
- **THEN** Gazel SHALL NOT expose registration or password flows
- **AND** SHALL NOT require a users or accounts table
- **AND** SHALL NOT evaluate roles, groups, permissions, or other claims for authorization

### Requirement: External URLs and provider HTTP behavior are explicit
Gazel MUST derive its callback URL only from the configured external origin and MUST NOT trust request host or forwarded headers for OIDC security decisions. Every outbound backend OIDC discovery, JWKS, and token request MUST use HTTPS except for HTTP loopback development endpoints and MUST NOT follow HTTP redirects.

#### Scenario: Callback behind a reverse proxy
- **WHEN** Gazel is reached through a reverse proxy with arbitrary `Host`, `Forwarded`, or `X-Forwarded-*` headers
- **THEN** the authorization request SHALL still use exactly the normalized configured external origin plus `/auth/callback`

#### Scenario: Insecure discovered endpoint
- **WHEN** discovery supplies an HTTP authorization, token, or JWKS endpoint on a non-loopback host
- **THEN** Gazel SHALL reject the provider configuration
- **AND** SHALL NOT send an authorization code or client secret to that endpoint

#### Scenario: Backend provider response redirects
- **WHEN** a backend OIDC discovery, JWKS, or token request receives an HTTP redirect
- **THEN** Gazel SHALL reject that backend request
- **AND** SHALL NOT follow the redirect

#### Scenario: Browser authorization redirect
- **WHEN** login produces a validated authorization endpoint URL
- **THEN** Gazel SHALL redirect the user agent to that provider URL as required by the Authorization Code flow
- **AND** the backend no-follow policy SHALL NOT prohibit this browser redirect
