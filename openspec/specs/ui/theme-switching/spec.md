## ADDED Requirements

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

The app SHALL persist the user's theme preference to `localStorage` under the key `gazel.theme`.

#### Scenario: User changes theme
- **WHEN** the user selects a theme preference (light, dark, or system)
- **THEN** the value SHALL be written to `localStorage` key `gazel.theme`
- **AND** the `data-theme` attribute on `<html>` SHALL update immediately

#### Scenario: Preference survives reload
- **WHEN** the user has set a theme preference and reloads the page
- **THEN** the previously selected theme SHALL be applied on load without flash

### Requirement: Runtime theme toggling

The theme store SHALL expose a function to change the theme at runtime, updating the DOM and persisting the preference.

#### Scenario: Toggle from light to dark
- **WHEN** `setTheme('dark')` is called
- **THEN** `data-theme` on `<html>` SHALL be set to `dark`
- **AND** `localStorage` key `gazel.theme` SHALL be set to `dark`
- **AND** all CSS custom properties from the dark theme SHALL take effect

#### Scenario: Set to system preference
- **WHEN** `setTheme('system')` is called
- **THEN** `localStorage` key `gazel.theme` SHALL be set to `system`
- **AND** `data-theme` SHALL reflect the current OS preference
- **AND** if the OS preference changes while the app is open, the theme SHALL update automatically

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
