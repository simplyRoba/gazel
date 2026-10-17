## MODIFIED Requirements

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

### Requirement: Enabled authentication uses five functional settings
The enabled authentication boundary SHALL consist of five functional settings: `GAZEL_AUTH_ENABLED=true` plus non-empty `GAZEL_EXTERNAL_URL`, `GAZEL_OIDC_ISSUER`, `GAZEL_OIDC_CLIENT_ID`, and `GAZEL_OIDC_CLIENT_SECRET`. The separately defined `GAZEL_OIDC_PROVIDER_NAME` SHALL remain optional display-only metadata. Every authentication value read while parsing enabled authentication SHALL be valid Unicode; present non-Unicode values SHALL fail configuration loading rather than use a default or behave as absent.

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
