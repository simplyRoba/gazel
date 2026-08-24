## ADDED Requirements

### Requirement: Authentication enablement is explicit
Built-in authentication SHALL be controlled by `GAZEL_AUTH_ENABLED`, which SHALL default to `false` when absent and SHALL accept only explicit boolean values.

#### Scenario: Authentication flag absent
- **WHEN** `GAZEL_AUTH_ENABLED` is not set
- **THEN** built-in authentication SHALL be disabled
- **AND** no authentication-specific configuration SHALL be required

#### Scenario: Authentication explicitly disabled
- **WHEN** `GAZEL_AUTH_ENABLED=false`
- **THEN** built-in authentication SHALL be disabled
- **AND** authentication-specific values SHALL NOT alter existing startup or request behavior

#### Scenario: Invalid authentication flag
- **WHEN** `GAZEL_AUTH_ENABLED` is present but is not `true` or `false`
- **THEN** configuration loading SHALL fail
- **AND** the application SHALL NOT start in an accidentally unauthenticated mode

### Requirement: Enabled authentication requires complete OIDC configuration
When `GAZEL_AUTH_ENABLED=true`, Gazel MUST require non-empty values for `GAZEL_AUTH_SECRET`, `GAZEL_EXTERNAL_URL`, `GAZEL_OIDC_ISSUER`, `GAZEL_OIDC_CLIENT_ID`, and `GAZEL_OIDC_CLIENT_SECRET`.

#### Scenario: Complete enabled configuration
- **WHEN** authentication is enabled and every required authentication variable is valid and non-empty
- **THEN** configuration loading SHALL produce an enabled OIDC configuration

#### Scenario: Missing enabled configuration value
- **WHEN** authentication is enabled and any required authentication variable is absent or empty
- **THEN** configuration loading SHALL fail with an error identifying the invalid variable
- **AND** the application SHALL NOT start

### Requirement: Authentication security values are validated at startup
`GAZEL_AUTH_SECRET` MUST decode from standard Base64 to at least 64 cryptographically random bytes. `GAZEL_EXTERNAL_URL` and `GAZEL_OIDC_ISSUER` MUST be absolute URLs without credentials, query strings, or fragments; HTTPS SHALL be required except for HTTP loopback development URLs. The external URL MUST represent a root-mounted origin without a non-root path and SHALL be normalized as an origin before callback construction.

#### Scenario: Valid HTTPS URLs and secret
- **WHEN** authentication is enabled with HTTPS external and issuer URLs and a Base64 secret decoding to at least 64 bytes
- **THEN** local configuration validation SHALL succeed
- **AND** the callback URL SHALL be the normalized external origin plus `/auth/callback`

#### Scenario: Valid loopback development URLs
- **WHEN** an enabled external URL or issuer URL uses HTTP with host `localhost`, `127.0.0.1`, or `::1`
- **THEN** local URL validation SHALL permit it for development and tests

#### Scenario: Weak or malformed auth secret
- **WHEN** `GAZEL_AUTH_SECRET` is invalid Base64 or decodes to fewer than 64 bytes
- **THEN** configuration loading SHALL fail
- **AND** the application SHALL NOT start

#### Scenario: Unsafe or ambiguous URL
- **WHEN** an enabled external or issuer URL uses unsupported HTTP on a non-loopback host, includes credentials, a query, or a fragment, or the external URL includes a non-root path
- **THEN** configuration loading SHALL fail
- **AND** the application SHALL NOT start
