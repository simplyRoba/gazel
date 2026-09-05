## Purpose

SQLite persistence with WAL mode, automatic database-file creation, and startup schema initialization and upgrades.

## Requirements

### Requirement: SQLite persistence uses WAL mode

The application SHALL use SQLite with Write-Ahead Logging as its persistence architecture.

#### Scenario: Database connection
- **WHEN** the application connects to its SQLite database
- **THEN** the journal mode SHALL be set to WAL

### Requirement: Database file auto-creation

The application SHALL create the SQLite database file and any missing parent directories on first startup.

#### Scenario: First run with non-existent database path
- **WHEN** the application starts and the configured database file does not exist
- **THEN** the parent directories SHALL be created if missing
- **AND** the SQLite database file SHALL be created automatically

#### Scenario: In-memory database path
- **WHEN** the database path is `:memory:`
- **THEN** the application SHALL NOT attempt to create parent directories
- **AND** the application SHALL use an in-memory SQLite database

### Requirement: Database schema is initialized and upgraded on startup

Before serving traffic, the application SHALL initialize a fresh SQLite database or automatically upgrade a supported older schema to the current schema. Schema upgrades SHALL preserve existing application data.

#### Scenario: Fresh database
- **WHEN** the application starts with a new empty database
- **THEN** the current schema SHALL be initialized before startup completes

#### Scenario: Supported older database
- **WHEN** the application starts with a supported older database schema
- **THEN** the schema SHALL be upgraded to the current version before traffic is served
- **AND** existing application data SHALL be preserved

#### Scenario: Current database
- **WHEN** the application starts with the current database schema
- **THEN** no schema changes SHALL be required
- **AND** startup SHALL proceed normally
