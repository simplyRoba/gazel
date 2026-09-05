## Purpose

Application configuration through environment variables, with defined defaults and validation behavior.

## Requirements

### Requirement: Environment-variable-based configuration

The application SHALL read configuration from environment variables prefixed with `GAZEL_`, falling back to sensible defaults when variables are not set.

#### Scenario: All defaults
- **WHEN** the application starts with no `GAZEL_*` environment variables set
- **THEN** the port SHALL default to `4110`
- **AND** the database path SHALL default to `/data/gazel.db`
- **AND** the log level SHALL default to `info`

#### Scenario: Custom port
- **WHEN** `GAZEL_PORT` is set to `9000`
- **THEN** the application SHALL use port `9000`

#### Scenario: Custom database path
- **WHEN** `GAZEL_DB_PATH` is set to `/tmp/test.db`
- **THEN** the application SHALL use `/tmp/test.db` as the database path

#### Scenario: Custom log level
- **WHEN** `GAZEL_LOG_LEVEL` is set to `debug`
- **THEN** the tracing subscriber SHALL use the `debug` level filter

#### Scenario: Invalid port value
- **WHEN** `GAZEL_PORT` is set to a non-numeric value
- **THEN** the application SHALL fall back to the default port `4110`

### Requirement: Authentication enablement is explicit
Built-in authentication SHALL be controlled by `GAZEL_AUTH_ENABLED`, which SHALL default to `false` when absent and SHALL accept only explicit boolean values.

#### Scenario: Authentication flag absent
- **WHEN** `GAZEL_AUTH_ENABLED` is not set
- **THEN** built-in authentication SHALL be disabled
- **AND** no authentication-specific configuration SHALL be required

#### Scenario: Authentication explicitly disabled
- **WHEN** `GAZEL_AUTH_ENABLED=false`
- **THEN** built-in authentication SHALL be disabled
- **AND** authentication-specific values SHALL NOT alter existing application or API behavior
- **AND** the separately defined inert `GET /auth/config` response SHALL remain available

#### Scenario: Invalid authentication flag
- **WHEN** `GAZEL_AUTH_ENABLED` is present but is not `true` or `false`
- **THEN** configuration loading SHALL fail
- **AND** the application SHALL NOT start in an accidentally unauthenticated mode

### Requirement: Enabled authentication uses five functional settings
The enabled authentication boundary SHALL consist of five functional settings: `GAZEL_AUTH_ENABLED=true` plus non-empty `GAZEL_EXTERNAL_URL`, `GAZEL_OIDC_ISSUER`, `GAZEL_OIDC_CLIENT_ID`, and `GAZEL_OIDC_CLIENT_SECRET`. The separately defined `GAZEL_OIDC_PROVIDER_NAME` SHALL remain optional display-only metadata.

#### Scenario: Complete five-setting configuration
- **WHEN** the enable flag and every required OIDC value are valid and non-empty
- **THEN** configuration loading SHALL produce an enabled OIDC configuration

#### Scenario: Missing enabled configuration value
- **WHEN** authentication is enabled and any required authentication variable is absent or empty
- **THEN** configuration loading SHALL fail with an error identifying the invalid variable
- **AND** the application SHALL NOT start

### Requirement: OIDC provider display name is configurable
`GAZEL_OIDC_PROVIDER_NAME` SHALL optionally configure the human-readable provider name shown by the public login page and SHALL default to `OpenID Connect`.

#### Scenario: Provider name absent
- **WHEN** authentication is enabled without `GAZEL_OIDC_PROVIDER_NAME`
- **THEN** the provider display name SHALL be `OpenID Connect`

#### Scenario: Custom provider name
- **WHEN** `GAZEL_OIDC_PROVIDER_NAME=Authentik`
- **THEN** the public auth config SHALL expose provider display name `Authentik`
- **AND** the value SHALL NOT affect issuer, client, or authorization decisions

#### Scenario: Invalid provider name
- **WHEN** the configured provider name is empty after trimming, exceeds 80 Unicode scalar values, or contains control characters
- **THEN** configuration loading SHALL fail
- **AND** the application SHALL NOT start with an unusable login label

### Requirement: Authentication URLs are validated at startup
`GAZEL_EXTERNAL_URL` and `GAZEL_OIDC_ISSUER` MUST be absolute URLs without credentials, query strings, or fragments; HTTPS SHALL be required except for HTTP loopback development URLs. The external URL MUST represent a root-mounted origin without a non-root path and SHALL be normalized as an origin before callback construction. The validated OIDC issuer identifier MUST preserve its exact configured textual form for exact comparison with provider metadata.

#### Scenario: Valid HTTPS URLs
- **WHEN** authentication is enabled with valid HTTPS external and issuer URLs
- **THEN** local configuration validation SHALL succeed
- **AND** the callback URL SHALL be the normalized external origin plus `/auth/callback`

#### Scenario: Valid loopback development URLs
- **WHEN** an enabled external URL or issuer URL uses HTTP with host `localhost`, `127.0.0.1`, or `::1`
- **THEN** local URL validation SHALL permit it for development and tests

#### Scenario: Exact origin-only issuer identifier
- **WHEN** `GAZEL_OIDC_ISSUER` is `https://id.example` and provider metadata reports `https://id.example`
- **THEN** discovery issuer validation SHALL succeed
- **AND** Gazel SHALL NOT normalize the configured issuer to `https://id.example/`

#### Scenario: Trailing slash remains significant
- **WHEN** `GAZEL_OIDC_ISSUER` is `https://id.example/` and provider metadata reports `https://id.example`
- **THEN** discovery issuer validation SHALL fail

#### Scenario: Unsafe or ambiguous URL
- **WHEN** an enabled external or issuer URL uses unsupported HTTP on a non-loopback host, includes credentials, a query, or a fragment, or the external URL includes a non-root path
- **THEN** configuration loading SHALL fail
- **AND** the application SHALL NOT start

### Requirement: Cookie encryption requires no operator-managed secret
Gazel SHALL NOT define or require an operator-managed authentication cookie secret setting when using process-local sessions; it SHALL generate the private-cookie key securely during each enabled startup.

#### Scenario: Enabled startup generates cookie key
- **WHEN** authentication is enabled with complete OIDC configuration
- **THEN** Gazel SHALL obtain a cryptographically secure random private-cookie key before serving traffic
- **AND** no operator-supplied cookie key SHALL be required

#### Scenario: Secure randomness unavailable
- **WHEN** Gazel cannot securely generate the private-cookie key
- **THEN** startup SHALL fail
- **AND** Gazel SHALL NOT fall back to a plaintext cookie or unauthenticated operation
