## Purpose

Theme switching: flash-free theme initialization, preference persistence, runtime toggling, system-preference change detection, and theme reconciliation on app init.

## Requirements

### Requirement: Theme initialization without flash

The app SHALL apply the correct theme (light or dark) before the first paint to prevent a flash of incorrect theme colors.

#### Scenario: Stored preference exists
- **WHEN** the app loads and `localStorage` contains a `gazel.theme` key with value `light` or `dark`
- **THEN** the `data-theme` attribute on `<html>` SHALL be set to that value before any content renders

#### Scenario: Stored preference is system
- **WHEN** the app loads and `localStorage` contains `gazel.theme` with value `system`
- **THEN** the `data-theme` attribute SHALL be set based on the OS `prefers-color-scheme` media query result

#### Scenario: No stored preference
- **WHEN** the app loads and no `gazel.theme` key exists in `localStorage`
- **THEN** the `data-theme` attribute SHALL be set based on the OS `prefers-color-scheme` media query result

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

### Requirement: System preference change detection

The theme store SHALL listen for OS-level color scheme changes and update the effective theme when the preference is set to `system`.

#### Scenario: OS switches to dark while app is open
- **WHEN** the theme preference is `system`
- **AND** the OS color scheme changes from light to dark
- **THEN** `data-theme` on `<html>` SHALL update to `dark` without page reload

#### Scenario: OS changes ignored when explicit preference set
- **WHEN** the theme preference is explicitly `light` or `dark`
- **AND** the OS color scheme changes
- **THEN** the `data-theme` attribute SHALL NOT change

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
