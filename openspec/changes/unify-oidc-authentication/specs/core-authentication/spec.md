## MODIFIED Requirements

### Requirement: OIDC provider discovery is fail-closed
When authentication is enabled, Gazel MUST perform OpenID Connect Discovery from the configured issuer during startup and MUST validate the discovered issuer, Authorization Code support, required endpoints, signing metadata, and associated JSON Web Key Set before serving traffic.

#### Scenario: Valid provider discovery
- **WHEN** the configured issuer returns valid standards-compliant discovery metadata
- **AND** discovery retrieves a JWKS containing at least one signature-verification key compatible with an advertised ID-token signing algorithm
- **THEN** Gazel SHALL initialize and retain its OIDC client and provider metadata
- **AND** startup SHALL continue

#### Scenario: Unreachable or malformed provider discovery
- **WHEN** discovery or associated JWKS retrieval fails, redirects, returns malformed data, returns a different issuer, omits a required endpoint, lacks Authorization Code support, returns an empty JWKS, or supplies unusable signing metadata
- **THEN** Gazel startup SHALL fail
- **AND** Gazel SHALL NOT fall back to unauthenticated operation

#### Scenario: Exact issuer and explicit code support
- **WHEN** metadata differs from the exact configured issuer text, lacks standalone `code` in response types, or advertises grant types without `authorization_code`
- **THEN** startup SHALL fail without serving traffic

#### Scenario: Unsigned algorithm advertised
- **WHEN** metadata advertises `none` among ID-token signing algorithms, even alongside a usable signed algorithm
- **THEN** startup SHALL reject that metadata

### Requirement: Login uses Authorization Code flow with state, nonce, and PKCE
For unauthenticated clients, `GET /auth/login` MUST initiate generic OIDC Authorization Code flow using a fresh state value, fresh nonce, and a SHA-256 PKCE challenge for each attempt. Gazel SHALL allow at most 1,024 pending login transactions, pruning expired entries before checking capacity.

#### Scenario: Login initiation
- **WHEN** an unauthenticated client requests `GET /auth/login`
- **THEN** Gazel SHALL create a short-lived backend login transaction bound to that client’s session
- **AND** the transaction SHALL contain the state, nonce, PKCE verifier, and validated local return target
- **AND** Gazel SHALL redirect to the startup-discovered authorization endpoint with `response_type=code`, the `openid` scope, configured client ID, configured external callback URL, state, nonce, PKCE challenge, and `code_challenge_method=S256`

#### Scenario: Fresh login security values
- **WHEN** the same unauthenticated client initiates two login attempts
- **THEN** each successfully saved attempt SHALL replace the prior login transaction unless capacity is saturated
- **AND** each authorization request SHALL use newly generated state, nonce, and PKCE values

#### Scenario: Login requested with a valid session
- **WHEN** an already authenticated client requests `GET /auth/login`
- **THEN** Gazel SHALL redirect to the validated local `return_to` target or `/`
- **AND** SHALL NOT invalidate or replace the authenticated session
- **AND** SHALL NOT initiate a provider authorization request

#### Scenario: Expired attempts free capacity
- **WHEN** login initiation checks the global limit of 1,024 pending transactions
- **THEN** transactions at least five minutes old SHALL be pruned before the capacity check
- **AND** a new pending transaction SHALL expire five minutes after creation

#### Scenario: Final pending slot is accepted
- **WHEN** 1,023 unexpired pending transactions remain after pruning and an unauthenticated browser starts a new login
- **THEN** Gazel SHALL accept the 1,024th transaction and redirect to provider authorization after saving its binding

#### Scenario: Consumed attempt frees capacity
- **WHEN** a pending transaction is consumed while capacity is full
- **THEN** a subsequent login SHALL be able to use the freed slot

#### Scenario: Saturation preserves all existing attempts
- **WHEN** 1,024 unexpired pending transactions exist and an unauthenticated browser requests login, including a browser replacing its own attempt
- **THEN** Gazel SHALL preserve every prior transaction and browser binding
- **AND** redirect with `303 See Other` to `/login?error=provider_unavailable&return_to=<encoded-safe-local-target>` using the validated requested target or `/`
- **AND** SHALL NOT initiate provider authorization for the new attempt

#### Scenario: Authenticated login at capacity
- **WHEN** a valid authenticated session requests login while pending capacity is full
- **THEN** Gazel SHALL redirect to its validated return target without allocating a transaction or changing the session

#### Scenario: Login binding cannot be saved
- **WHEN** a login initiation cannot save its browser binding or replacement
- **THEN** Gazel SHALL NOT redirect to provider authorization or leave a usable orphan attempt
- **AND** SHALL preserve the prior valid attempt and binding unless a newer login or logout has invalidated them
- **AND** SHALL redirect with `303 See Other` and `provider_unavailable` with the validated requested target or `/`

### Requirement: Return navigation cannot become an open redirect
Gazel SHALL form-decode the `return_to` query value once to obtain a serialized local protected UI target, limit that target to 2,048 UTF-8 bytes before validation-only percent decoding, and redirect using the original accepted serialization. Targets SHALL have exactly one leading slash and obey the navigation validation scenarios below. Invalid targets SHALL become `/` before storage or redirect. The backend SHALL NOT infer browser fragments; the SPA SHALL carry any fragment as query-parameter data.

#### Scenario: Backend navigation preserves only path and query
- **WHEN** authentication middleware handles an unauthenticated request target `/settings?tab=data`
- **THEN** it SHALL redirect to `/login?return_to=%2Fsettings%3Ftab%3Ddata`
- **AND** it SHALL NOT append, infer, or claim to have received a browser URL fragment

#### Scenario: SPA-originated hash is query-parameter data
- **WHEN** the wire request is `GET /auth/login?return_to=%2Fsettings%3Ftab%3Ddata%23export`
- **THEN** `%23export` SHALL be treated as data inside the `return_to` query parameter, not as a fragment of the `/auth/login` request
- **AND** Gazel SHALL decode and validate the serialized local target `/settings?tab=data#export`
- **AND** store that target only in the backend login transaction
- **AND** redirect to the stored target after successful authentication

#### Scenario: External or reserved return target
- **WHEN** the decoded login value is external, protocol-relative, malformed, reserved, contains a backslash, or contains a control character
- **THEN** Gazel SHALL replace it with `/`
- **AND** SHALL NOT reflect the unsafe value into a `Location` header

#### Scenario: Return target absent
- **WHEN** login receives no `return_to`
- **THEN** the backend login transaction SHALL use `/`

#### Scenario: Return target exceeds the length limit
- **WHEN** the serialized return target obtained by one query form-decoding exceeds 2,048 UTF-8 bytes
- **THEN** Gazel SHALL replace it with `/`
- **AND** authentication middleware SHALL use `/login?return_to=%2F` instead of reflecting the oversized target
- **AND** `/auth/login` SHALL store only `/` in the backend login transaction

#### Scenario: Serialized size is measured before validation decoding
- **WHEN** a target contains valid escapes that make its serialized form exceed 2,048 UTF-8 bytes although its validation-decoded form is shorter
- **THEN** Gazel SHALL use `/`
- **AND** wire query encoding size alone SHALL NOT cause rejection of a target whose serialized form is within the limit

#### Scenario: Ambiguous path or invalid encoding
- **WHEN** a target contains malformed percent escapes or UTF-8, raw or decoded backslashes or Unicode controls U+0000–U+001F or U+007F–U+009F, dot path segments, encoded path separators `/`, `\`, `?`, or `#`, or residual percent or nested escapes in its path
- **THEN** Gazel SHALL use `/`

#### Scenario: Reserved route prefixes
- **WHEN** a target path names `/auth`, `/login`, `/api`, or `/health`, or a descendant or encoded equivalent of those paths
- **THEN** Gazel SHALL use `/`
- **AND** paths such as `/apiary` SHALL NOT be rejected merely for sharing a prefix
- **AND** `/uploads` SHALL NOT be rejected solely for that namespace in Gazel

#### Scenario: Valid query and fragment escapes retain serialization
- **WHEN** a safe target is `/settings?label=100%25&next=a%2Fb#part%20one`
- **THEN** Gazel SHALL accept and retain that exact serialized target
- **AND** SHALL NOT recursively decode, normalize, or reject its query merely because validation decoding yields a literal percent

### Requirement: Callback validation is one-time and complete
`GET /auth/callback` MUST validate and atomically consume the backend login transaction before establishing a Gazel session, so at most one concurrent callback can obtain it. A successful callback SHALL redirect to the safe `return_to` stored in that transaction. Every failed callback SHALL instead respond with `303 See Other` and redirect to `/login?error=<stable-error-code>&return_to=<encoded-safe-local-target>`. The `return_to` parameter MUST always be present; when no validated transaction target remains, Gazel SHALL use `/`, encoded as `%2F`.

#### Scenario: Valid callback
- **WHEN** the callback contains an authorization code and state matching an unexpired login transaction
- **AND** the provider accepts the matching PKCE verifier and selected token client-authentication method
- **AND** the token response contains a valid ID token whose signature, raw issuer, audience, required positive issued-at value, expiration, nonce, and authorized-party claim when present all validate
- **AND** any access-token hash claim in the ID token matches the returned access token
- **THEN** Gazel SHALL rotate the session identifier
- **AND** establish an authenticated Gazel session
- **AND** redirect the browser to the safe `return_to` stored in the consumed login transaction

#### Scenario: Missing or mismatched state
- **WHEN** callback state is absent, does not match the browser-bound transaction, or has no corresponding transaction
- **THEN** Gazel SHALL NOT contact the token endpoint or establish a session
- **AND** SHALL redirect to `/login?error=authentication_failed&return_to=%2F`

#### Scenario: Expired or replayed login transaction
- **WHEN** a callback uses an expired transaction or reuses a transaction already consumed by a valid browser-bound callback
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
- **WHEN** a valid provider callback reports an OAuth error, the token endpoint returns a parsed OAuth rejection from a non-5xx response, or a successfully parsed token response fails token protocol or claim validation
- **THEN** Gazel SHALL NOT expose the provider’s description, callback parameters, or token details
- **AND** SHALL NOT establish a session
- **AND** SHALL redirect to `/login?error=authentication_failed&return_to=<encoded-safe-local-target>`

#### Scenario: Provider temporarily unavailable
- **WHEN** discovery-cached token or JWKS communication has a transport failure, HTTP 5xx response, unparseable response, or unexpected service response rather than a usable protocol response
- **THEN** Gazel SHALL NOT establish a session
- **AND** SHALL redirect to `/login?error=provider_unavailable&return_to=<encoded-safe-local-target>`

#### Scenario: Malformed callback is rejected before token I/O
- **WHEN** callback query form decoding encounters malformed percent encoding or UTF-8, duplicate `state`, `code`, `error`, `error_description`, or `error_uri`, empty success state or code, or success parameters mixed with error or error-detail parameters
- **THEN** Gazel SHALL reject it before token I/O with `authentication_failed` and fallback target `/`
- **AND** SHALL NOT consume a valid pending transaction

#### Scenario: Error details without a provider error
- **WHEN** a callback contains `error_description` or `error_uri` without a non-empty `error`, or an error callback has no non-empty matching state
- **THEN** Gazel SHALL reject it as malformed without token I/O or consuming a valid transaction

#### Scenario: Distinct provider error details are accepted and ignored
- **WHEN** a well-formed callback contains a non-empty `error`, matching browser-bound state, and one each of both `error_description` and `error_uri`, with no code
- **THEN** Gazel SHALL consume the matching transaction exactly once without token exchange
- **AND** SHALL redirect with `authentication_failed` and its stored safe return target
- **AND** SHALL ignore both detail values rather than reflect or fetch them

#### Scenario: Wrong browser cannot consume another browser's attempt
- **WHEN** a callback supplies an existing state without the valid matching browser binding
- **THEN** Gazel SHALL NOT exchange the code, authenticate, or consume that transaction
- **AND** the correctly bound browser SHALL still be able to complete it once

### Requirement: Local logout always invalidates the Gazel session
`POST /auth/logout` MUST invalidate the presented backend Gazel session and associated pending or in-flight login attempt, and expire its browser cookie without requiring provider logout support.

#### Scenario: Authenticated logout
- **WHEN** a client posts to `/auth/logout` with a valid Gazel session
- **THEN** Gazel SHALL delete the server-managed session
- **AND** expire the session cookie
- **AND** redirect with `303 See Other` to `/login?logged_out=1`
- **AND** subsequent use of the former cookie SHALL be unauthenticated

#### Scenario: Logout without a valid session
- **WHEN** a client posts to `/auth/logout` without a valid Gazel session
- **THEN** Gazel SHALL still expire any applicable cookie
- **AND** redirect with `303 See Other` to `/login?logged_out=1`

#### Scenario: Logout does not automatically reauthenticate
- **WHEN** the browser follows the logout redirect
- **THEN** the public Svelte login page SHALL show its signed-out state
- **AND** Gazel SHALL wait for the user to activate the OIDC button

#### Scenario: Logout races with callback completion
- **WHEN** successful logout invalidates a presented session or its associated attempt while a callback is in flight
- **THEN** that callback SHALL NOT establish an authenticated session even if the provider later returns valid tokens
- **AND** subsequent protected requests with the former cookie SHALL remain unauthenticated
- **AND** this guarantee SHALL NOT require cancellation of a provider request already sent

#### Scenario: Logout persistence failure
- **WHEN** Gazel cannot persist session or associated-attempt invalidation
- **THEN** logout SHALL return HTTP `500` and expire the browser cookie
- **AND** SHALL NOT return the successful signed-out redirect or claim successful invalidation

### Requirement: External URLs and provider HTTP behavior are explicit
Gazel MUST derive its callback URL only from the configured external origin and MUST NOT trust request host or forwarded headers for OIDC security decisions. Every outbound backend OIDC URL SHALL be validated before any request, including initial JWKS retrieval. Discovery, JWKS, and token requests MUST use HTTPS except for Gazel’s HTTP loopback development endpoints, SHALL reject URL credentials and fragments, and MUST NOT follow HTTP redirects.

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

#### Scenario: Unsafe URL is rejected before initial retrieval
- **WHEN** startup metadata supplies a JWKS or other OIDC endpoint with credentials, a fragment, or a non-loopback HTTP URL
- **THEN** Gazel SHALL reject it before making any request to that URL, including the initial JWKS request
- **AND** SHALL fail startup without falling back to unauthenticated operation

## ADDED Requirements

### Requirement: ID-token claims are explicitly bound to the configured client
Gazel SHALL compare the raw issuer claim exactly with the configured issuer, require a positive `iat`, and require any `azp` to equal the client ID. It SHALL retain rejection of additional or untrusted audiences and verification of signature, expiry, nonce, and any `at_hash` without imposing an additional arbitrary freshness or leeway policy.

#### Scenario: Issuer spelling is significant
- **WHEN** an otherwise valid ID token has an issuer differing from the configured text, including only a trailing slash
- **THEN** Gazel SHALL reject authentication without refreshing signing keys

#### Scenario: Issued-at is required and positive
- **WHEN** an otherwise valid token omits `iat` or supplies zero, a negative value, or a malformed value
- **THEN** Gazel SHALL reject authentication

#### Scenario: Authorized party must match
- **WHEN** a token includes `azp` different from the configured client ID
- **THEN** Gazel SHALL reject it even if its audience contains that client ID

#### Scenario: Additional audiences remain rejected
- **WHEN** a token includes additional or untrusted audiences
- **THEN** Gazel SHALL reject it even if `azp` matches the client ID

### Requirement: Authentication persistence cannot resurrect invalidated attempts
Gazel SHALL save authenticated state only after token verification and identifier rotation, clear preauthentication fields, and persist before redirecting to success. Storage failures SHALL NOT establish valid authenticated state. Session and binding updates SHALL NOT overwrite a newer login or resurrect an attempt invalidated by logout or replacement.

#### Scenario: Authenticated save fails
- **WHEN** verified callback authentication cannot be saved after session rotation
- **THEN** neither the old nor proposed new identifier SHALL grant authenticated access from that callback
- **AND** Gazel SHALL redirect with `303 See Other`, `provider_unavailable`, and the transaction’s safe return target
- **AND** SHALL NOT issue a success redirect

#### Scenario: Successful save clears pending state
- **WHEN** a callback successfully persists the rotated authenticated session
- **THEN** the session SHALL no longer carry usable preauthentication state or binding
- **AND** the browser SHALL receive its success redirect only after that persistence succeeds

#### Scenario: Replacement invalidates an in-flight callback
- **WHEN** a newer login successfully replaces the browser’s attempt before its earlier callback completes
- **THEN** the earlier callback SHALL NOT establish authentication or overwrite the newer binding even if its token response is valid
- **AND** the newer attempt SHALL remain usable once
- **AND** a provider request already sent SHALL NOT need to be cancelled

#### Scenario: Failed older save cannot overwrite a newer login
- **WHEN** an older login initiation or callback fails persistence after a newer login has saved its binding
- **THEN** failure handling SHALL preserve the newer login and SHALL NOT restore an older binding over it

### Requirement: Authentication responses prohibit caching
Every response to `/auth` or `/auth/*` SHALL include `Cache-Control: no-store` in enabled and disabled modes, including errors, unknown routes, and unsupported methods. Authentication-required API `401` responses and protected-document login redirects SHALL also include `Cache-Control: no-store`.

#### Scenario: All authentication endpoint responses are non-cacheable
- **WHEN** a client receives any `/auth` or `/auth/*` response, including config, login, callback, logout, `404`, or `405` in either authentication mode
- **THEN** that response SHALL include `Cache-Control: no-store`

#### Scenario: Authentication gate responses are non-cacheable
- **WHEN** an unauthenticated protected request receives an authentication-required API `401` or document redirect to login
- **THEN** that response SHALL include `Cache-Control: no-store`

### Requirement: Authentication diagnostics expose only fixed safe context
Authentication diagnostics SHALL use fixed operation and category labels and an HTTP status when available. They SHALL NOT expose provider bodies, arbitrary validation strings, secrets, tokens, callback parameters, or URL query data. User-facing failures SHALL retain only the stable local error code and validated return target.

#### Scenario: Sensitive provider failure details
- **WHEN** provider communication or validation fails with a body, URL query, token, or arbitrary error description containing sensitive text
- **THEN** diagnostic output SHALL contain only fixed operation/category context and available HTTP status
- **AND** the browser failure SHALL NOT expose that text
