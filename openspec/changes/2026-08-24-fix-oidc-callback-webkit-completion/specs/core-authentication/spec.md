## MODIFIED Requirements

### Requirement: Callback validation is one-time and complete
The successful-response portion of this requirement is modified: after a callback has succeeded under the existing canonical validation requirements and established the authenticated Gazel session, Gazel SHALL return a minimal `200 OK` HTML completion document instead of returning `303 See Other` directly to `return_to`. All callback validation and failed-callback behavior remain as defined by the canonical requirement's existing scenarios, referenced below. The canonical `Return navigation cannot become an open redirect` requirement continues to govern acceptance, storage, and fallback of `return_to` without modification.

#### Scenario: Valid callback
- **GIVEN** an OIDC callback has satisfied the canonical valid-callback conditions and established an authenticated Gazel session
- **WHEN** Gazel emits the successful callback response
- **THEN** the response SHALL be `200 OK` with an HTML content type
- **AND** it SHALL include `Cache-Control: no-store` and `Referrer-Policy: no-referrer`
- **AND** it SHALL NOT include a `Location` header
- **AND** the authenticated session cookie SHALL retain `SameSite=Lax`
- **AND** the minimal completion document SHALL perform a fresh same-origin navigation to the already validated `return_to` using `location.replace()` or equivalent behavior
- **AND** any script and HTML fallback representations of `return_to` SHALL be safely escaped for their output contexts

#### Scenario: Missing or mismatched state
- **WHEN** the canonical missing-or-mismatched-state scenario occurs
- **THEN** its existing failed-callback behavior SHALL remain unchanged

#### Scenario: Expired or replayed login transaction
- **WHEN** the canonical expired-or-replayed-transaction scenario occurs
- **THEN** its existing failed-callback behavior SHALL remain unchanged

#### Scenario: Concurrent callback replay
- **WHEN** the canonical concurrent-callback-replay scenario occurs
- **THEN** its existing failed-callback behavior SHALL remain unchanged

#### Scenario: Invalid nonce or ID token
- **WHEN** the canonical invalid-nonce-or-ID-token scenario occurs
- **THEN** its existing failed-callback behavior SHALL remain unchanged

#### Scenario: Provider authorization or validation error
- **WHEN** the canonical provider-authorization-or-validation-error scenario occurs
- **THEN** its existing failed-callback behavior SHALL remain unchanged

#### Scenario: Provider temporarily unavailable
- **WHEN** the canonical provider-unavailable scenario occurs
- **THEN** its existing failed-callback behavior SHALL remain unchanged

#### Scenario: Completion document contains no authentication material
- **GIVEN** a successful callback request contains OIDC callback parameters and results in an authenticated session
- **WHEN** Gazel returns the completion document
- **THEN** the document SHALL NOT expose the authorization code, state, nonce, PKCE verifier, access token, ID token, client secret, cookie or session identifier, backend session contents, or other authentication material
- **AND** the subsequent navigation SHALL NOT disclose the callback URL or its authentication parameters as referrer metadata
