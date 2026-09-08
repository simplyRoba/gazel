## Purpose

Defines flash-free theme initialization, light/dark/system preferences, persistence, system-theme changes, and preference precedence.

## Requirements

### Requirement: Theme initializes without a flash

The application SHALL apply the effective light or dark theme before content is first painted.

#### Scenario: Explicit preference is available
- **WHEN** an explicit light or dark preference is available at startup
- **THEN** that theme SHALL be applied before content renders

#### Scenario: System preference is selected
- **WHEN** the saved preference is `system`
- **THEN** the current operating-system color preference SHALL determine the initial theme
- **AND** the effective theme SHALL be applied before content renders

#### Scenario: No preference is available
- **WHEN** no saved theme preference is available at startup
- **THEN** the current operating-system color preference SHALL determine the initial theme

### Requirement: Theme preference persists

Selecting light, dark, or system SHALL update the effective theme immediately and persist the preference for future application sessions.

#### Scenario: User changes theme
- **WHEN** the user selects a theme preference
- **THEN** the effective theme SHALL update immediately
- **AND** the preference SHALL be saved as an application setting
- **AND** it SHALL be available early enough on the next startup to prevent an incorrect-theme flash

#### Scenario: Saving the preference fails
- **WHEN** the selected theme cannot be saved as an application setting
- **THEN** the previous theme preference SHALL be restored immediately
- **AND** a user-facing error SHALL be shown

#### Scenario: Page reload
- **WHEN** the user reloads after selecting a theme
- **THEN** the previously selected theme SHALL be applied before content renders

### Requirement: System preference remains reactive

System mode SHALL follow operating-system color preference changes while the application is open.

#### Scenario: System changes to dark
- **WHEN** the preference is `system`
- **AND** the operating system changes from light to dark
- **THEN** the application SHALL switch to dark without a page reload

#### Scenario: Explicit preference ignores system changes
- **WHEN** the preference is explicitly light or dark
- **AND** the operating-system color preference changes
- **THEN** the application's effective theme SHALL remain unchanged

### Requirement: Saved theme precedence

After startup, the application-wide saved preference SHALL normally be authoritative over any device-cached preference used for flash-free initial rendering.

#### Scenario: Preferences agree
- **WHEN** the application-wide and device-cached preferences agree
- **THEN** the effective theme SHALL remain unchanged

#### Scenario: Preferences disagree
- **WHEN** the application-wide preference differs from the device-cached preference
- **THEN** the application-wide preference SHALL become effective
- **AND** the device-cached preference SHALL be updated for future flash-free startup

#### Scenario: Existing explicit preference predates application settings
- **WHEN** preference synchronization occurs for the first time
- **AND** the application-wide preference is the default `system`
- **AND** the device already has an explicit light or dark preference from before application settings existed
- **THEN** the existing explicit preference SHALL remain effective
- **AND** it SHALL become the application-wide saved preference
- **AND** later initializations SHALL use normal application-wide precedence

#### Scenario: Application settings are unavailable
- **WHEN** application-wide settings cannot be loaded during initialization
- **THEN** the theme already applied for flash-free startup SHALL remain effective
- **AND** precedence reconciliation SHALL wait until a later initialization
