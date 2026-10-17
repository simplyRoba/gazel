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
- **THEN** the application's logging threshold SHALL be `debug`

#### Scenario: Invalid port value
- **WHEN** `GAZEL_PORT` is set to a non-numeric value
- **THEN** the application SHALL fall back to the default port `4110`

### Requirement: Authentication enablement is explicit
Built-in authentication SHALL be controlled by `GAZEL_AUTH_ENABLED`, which SHALL default to `false` when absent and SHALL accept only explicit boolean values. A present non-Unicode enable flag SHALL fail configuration loading rather than behave as absent. When absent or explicitly false, other OIDC settings SHALL be ignored, including invalid or non-Unicode values.

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

#### Scenario: Non-Unicode enable flag
- **WHEN** `GAZEL_AUTH_ENABLED` is present but cannot be read as Unicode
- **THEN** configuration loading SHALL fail
- **AND** Gazel SHALL NOT silently start without authentication

#### Scenario: Disabled authentication ignores other OIDC settings
- **WHEN** `GAZEL_AUTH_ENABLED` is absent or explicitly `false` and other authentication settings are invalid or non-Unicode
- **THEN** authentication SHALL remain disabled without validating those other settings
- **AND** existing application behavior and the inert public auth-config response SHALL remain unchanged

### Requirement: Enabled authentication requires five settings
The enabled authentication boundary SHALL require five settings: `GAZEL_AUTH_ENABLED=true` plus non-empty `GAZEL_EXTERNAL_URL`, `GAZEL_OIDC_ISSUER`, `GAZEL_OIDC_CLIENT_ID`, and `GAZEL_OIDC_CLIENT_SECRET`. The separately defined `GAZEL_OIDC_PROVIDER_NAME` SHALL remain optional display-only metadata, and `GAZEL_OIDC_AUTO_REDIRECT` SHALL be an optional login behavior flag. Every authentication value read while parsing enabled authentication SHALL be valid Unicode; present non-Unicode values SHALL fail configuration loading rather than use a default or behave as absent.

#### Scenario: Complete five-setting configuration
- **WHEN** the enable flag and every required OIDC value are valid and non-empty
- **THEN** configuration loading SHALL produce an enabled OIDC configuration

#### Scenario: Missing enabled configuration value
- **WHEN** authentication is enabled and any required authentication variable is absent or empty
- **THEN** configuration loading SHALL fail with an error identifying the invalid variable
- **AND** the application SHALL NOT start

#### Scenario: Enabled authentication rejects non-Unicode values
- **WHEN** authentication is enabled and any authentication setting read by the enabled parser, including optional provider display metadata, is present but not Unicode
- **THEN** configuration loading SHALL fail with an error identifying the variable without exposing its value
- **AND** the application SHALL NOT start

#### Scenario: Unrelated lenient configuration remains unchanged
- **WHEN** enabled authentication is valid but an unrelated setting has an invalid value for which existing configuration defines a fallback
- **THEN** authentication Unicode validation SHALL NOT change that setting’s existing fallback behavior

### Requirement: Automatic OIDC login is optional
When authentication is enabled, `GAZEL_OIDC_AUTO_REDIRECT` SHALL default to `false` when absent and SHALL accept only the exact strings `true` and `false`. Invalid or non-Unicode values SHALL fail startup with a safe error identifying the variable without exposing its value. When authentication is disabled, the flag SHALL NOT be read or validated.

#### Scenario: Manual login by default
- **WHEN** authentication is enabled and `GAZEL_OIDC_AUTO_REDIRECT` is absent or `false`
- **THEN** public auth config SHALL report `auto_redirect: false`

#### Scenario: Automatic login enabled
- **WHEN** authentication is enabled and `GAZEL_OIDC_AUTO_REDIRECT=true`
- **THEN** public auth config SHALL report `auto_redirect: true`

#### Scenario: Invalid automatic-login flag
- **WHEN** authentication is enabled and `GAZEL_OIDC_AUTO_REDIRECT` is non-Unicode or not exactly `true` or `false`, including an empty value
- **THEN** startup SHALL fail safely without exposing the value or disabling authentication

#### Scenario: Disabled authentication ignores automatic-login flag
- **WHEN** authentication is disabled and `GAZEL_OIDC_AUTO_REDIRECT` is invalid or non-Unicode
- **THEN** startup SHALL proceed without reading or validating the flag
- **AND** public auth config SHALL remain exactly `{ "enabled": false }`

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
