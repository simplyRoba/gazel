## ADDED Requirements

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

### Requirement: Testable configuration abstraction

Configuration reading SHALL be abstracted behind a `ConfigSource` trait, allowing tests to provide configuration without modifying real environment variables.

#### Scenario: Production configuration source
- **WHEN** the application starts in production
- **THEN** the `EnvConfigSource` SHALL read values from `std::env::var`

#### Scenario: Test configuration source
- **WHEN** a test needs to verify configuration behavior
- **THEN** a mock `ConfigSource` backed by a `HashMap` SHALL be usable
- **AND** the configuration parser SHALL produce identical results regardless of the source implementation
