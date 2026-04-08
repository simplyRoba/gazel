## ADDED Requirements

### Requirement: SQLite connection pool with WAL mode

The application SHALL create a SQLite connection pool configured with Write-Ahead Logging and appropriate settings for a single-user workload.

#### Scenario: Pool configuration
- **WHEN** the database pool is created
- **THEN** the journal mode SHALL be set to WAL
- **AND** the busy timeout SHALL be set to 5 seconds
- **AND** the maximum connections SHALL be set to 5

### Requirement: Database file auto-creation

The application SHALL create the SQLite database file and any missing parent directories on first startup.

#### Scenario: First run with non-existent database path
- **WHEN** the application starts and the configured database file does not exist
- **THEN** the parent directories SHALL be created if missing
- **AND** the SQLite database file SHALL be created automatically

#### Scenario: In-memory database path
- **WHEN** the database path is `:memory:`
- **THEN** the application SHALL NOT attempt to create parent directories
- **AND** the pool SHALL use an in-memory database

### Requirement: Migrations run on startup

The application SHALL execute all pending SQLite migrations automatically when the database pool is created.

#### Scenario: Fresh database
- **WHEN** the application connects to a database with no migration history
- **THEN** all migrations SHALL be applied in order

#### Scenario: Partially migrated database
- **WHEN** the application connects to a database with some migrations already applied
- **THEN** only the pending migrations SHALL be applied

#### Scenario: Fully migrated database
- **WHEN** the application connects to a database with all migrations already applied
- **THEN** no migrations SHALL be applied
- **AND** startup SHALL proceed normally

### Requirement: Bootstrap migration

An initial empty migration SHALL exist to bootstrap the migration tracking table.

#### Scenario: Initial migration exists
- **WHEN** the migrations directory is inspected
- **THEN** a timestamp-prefixed initial migration file SHALL exist
- **AND** the migration file SHALL contain no schema changes
