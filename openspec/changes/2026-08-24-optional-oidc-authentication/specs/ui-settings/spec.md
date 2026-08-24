## ADDED Requirements

### Requirement: Settings exposes local logout when authentication is enabled
The settings page SHALL provide a translated logout action only when the authenticated app-info response reports that built-in authentication is enabled.

#### Scenario: Authentication-enabled settings page
- **WHEN** `/api/info` includes `auth_enabled: true`
- **THEN** the settings page SHALL display an Authentication section
- **AND** SHALL display a translated Sign out action
- **AND** the action SHALL submit `POST /auth/logout` as a top-level browser form navigation

#### Scenario: Authentication-disabled settings page
- **WHEN** `/api/info` omits `auth_enabled`
- **THEN** the settings page SHALL NOT display the Authentication section or logout action
- **AND** the existing disabled-mode settings UI SHALL remain unchanged

#### Scenario: Successful local logout
- **WHEN** the user activates Sign out
- **THEN** the browser SHALL submit `POST /auth/logout`
- **AND** follow the response to `/login?logged_out=1`
- **AND** the public login page SHALL not automatically initiate provider login

### Requirement: App info signals enabled authentication without changing disabled output
The frontend `AppInfo` type SHALL accept an optional `auth_enabled` field whose value is present and `true` only in the enabled authenticated application.

#### Scenario: Disabled app-info compatibility
- **WHEN** built-in authentication is disabled
- **THEN** `/api/info` SHALL retain its existing response fields without adding `auth_enabled`

#### Scenario: Enabled app-info response
- **WHEN** built-in authentication is enabled and an authenticated client requests `/api/info`
- **THEN** the response SHALL include `auth_enabled: true`
