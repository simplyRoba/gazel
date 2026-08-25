## ADDED Requirements

### Requirement: Successful callback uses a completion document
After a callback has succeeded under the canonical `Callback validation is one-time and complete` requirement and established the authenticated Gazel session, Gazel SHALL return a minimal `200 OK` HTML completion document instead of returning `303 See Other` directly to `return_to`. For successful callbacks, references in the canonical requirements to redirecting or navigating the browser to `return_to` SHALL mean the client-side navigation performed by this completion document, not an HTTP `3xx` response. The canonical callback requirement remains authoritative for all callback validation and failed-callback behavior. The canonical `Return navigation cannot become an open redirect` requirement continues to govern acceptance, storage, and fallback of `return_to` without modification.

#### Scenario: Valid callback
- **GIVEN** an OIDC callback has satisfied the canonical valid-callback conditions and established an authenticated Gazel session
- **WHEN** Gazel emits the successful callback response
- **THEN** the response SHALL be `200 OK` with an HTML content type
- **AND** it SHALL include `Cache-Control: no-store` and `Referrer-Policy: no-referrer`
- **AND** it SHALL NOT include a `Location` header
- **AND** the authenticated session cookie SHALL retain `SameSite=Lax`
- **AND** the minimal completion document SHALL perform a fresh same-origin navigation to the already validated `return_to` using `location.replace()` or equivalent behavior
- **AND** any script and HTML fallback representations of `return_to` SHALL be safely escaped for their output contexts

#### Scenario: Completion document contains no authentication material
- **GIVEN** a successful callback request contains OIDC callback parameters and results in an authenticated session
- **WHEN** Gazel returns the completion document
- **THEN** the document SHALL NOT expose the authorization code, state, nonce, PKCE verifier, access token, ID token, client secret, cookie or session identifier, backend session contents, or other authentication material
- **AND** the subsequent navigation SHALL NOT disclose the callback URL or its authentication parameters as referrer metadata
