## MODIFIED Requirements

### Requirement: Theme preference persistence

The app SHALL persist the user's theme preference to `localStorage` under the key `gazel.theme` AND to the server via `PUT /api/settings`.

#### Scenario: User changes theme

- **WHEN** the user selects a theme preference (light, dark, or system)
- **THEN** the value SHALL be written to `localStorage` key `gazel.theme` synchronously
- **AND** the `data-theme` attribute on `<html>` SHALL update immediately
- **AND** an async `PUT /api/settings` request SHALL be sent with `{ "color_mode": "<value>" }`
- **AND** if the API call fails, the `localStorage` value SHALL remain (the server will be reconciled on next init)

#### Scenario: Preference survives reload

- **WHEN** the user has set a theme preference and reloads the page
- **THEN** the previously selected theme SHALL be applied on load without flash via the inline `localStorage` read

### Requirement: Runtime theme toggling

The theme store SHALL expose a function to change the theme at runtime, updating the DOM, persisting to `localStorage`, and syncing to the server.

#### Scenario: Toggle from light to dark

- **WHEN** `setTheme('dark')` is called
- **THEN** `data-theme` on `<html>` SHALL be set to `dark`
- **AND** `localStorage` key `gazel.theme` SHALL be set to `dark`
- **AND** all CSS custom properties from the dark theme SHALL take effect
- **AND** `PUT /api/settings` SHALL be called with `{ "color_mode": "dark" }`

#### Scenario: Set to system preference

- **WHEN** `setTheme('system')` is called
- **THEN** `localStorage` key `gazel.theme` SHALL be set to `system`
- **AND** `data-theme` SHALL reflect the current OS preference
- **AND** if the OS preference changes while the app is open, the theme SHALL update automatically
- **AND** `PUT /api/settings` SHALL be called with `{ "color_mode": "system" }`

## ADDED Requirements

### Requirement: Theme reconciliation on app init

On app startup, the theme store SHALL reconcile the `localStorage` value (applied by the inline script) with the server-stored `color_mode` from the settings API.

#### Scenario: Server and localStorage agree

- **WHEN** the settings store fetches settings from the API
- **AND** `server.color_mode` matches `localStorage.gazel.theme`
- **THEN** no additional action SHALL be taken

#### Scenario: Server and localStorage disagree

- **WHEN** the settings store fetches settings from the API
- **AND** `server.color_mode` differs from `localStorage.gazel.theme`
- **THEN** the server value SHALL be treated as authoritative
- **AND** `localStorage.gazel.theme` SHALL be updated to match the server value
- **AND** the `data-theme` attribute SHALL be updated if the effective theme changes

#### Scenario: First sync from existing localStorage

- **WHEN** the settings store fetches settings from the API for the first time
- **AND** `server.color_mode` is `system` (the default)
- **AND** `localStorage.gazel.theme` contains an explicit `light` or `dark` value set before the settings API existed
- **THEN** the localStorage value SHALL be pushed to the server via `PUT /api/settings`
- **AND** subsequent inits SHALL treat the server as authoritative

#### Scenario: API unavailable during init

- **WHEN** the settings store fails to fetch settings from the API
- **THEN** the theme SHALL remain as applied by the inline `localStorage` script
- **AND** no reconciliation SHALL occur
