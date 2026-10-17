## MODIFIED Requirements

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

#### Scenario: Login remains an explicit user action
- **WHEN** the public login page finishes loading, including a failure or retry state
- **THEN** it SHALL wait for the user to activate the provider button rather than automatically navigate to provider authorization
