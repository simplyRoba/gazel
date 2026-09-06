## Purpose

Defines application preference behavior, the settings page, navigation, import/export controls, and authentication settings.

## Requirements

### Requirement: Application preferences

Current unit system, distance unit, volume unit, currency, color mode, and locale preferences SHALL apply consistently across the protected application.

#### Scenario: Preference update begins

- **WHEN** the user changes a preference
- **THEN** the new value SHALL apply immediately while it is being saved

#### Scenario: Preference update succeeds

- **WHEN** saving a preference succeeds
- **THEN** the new value SHALL remain selected during later navigation and future settings reads

#### Scenario: Preference update fails

- **WHEN** saving a preference fails
- **THEN** the previous value SHALL be restored
- **AND** a user-facing error SHALL be shown

#### Scenario: Initial settings request fails

- **WHEN** current preferences cannot be loaded during initialization
- **THEN** the application SHALL remain functional using defaults of `metric`, `km`, `l`, `EUR`, `system`, and `en`

### Requirement: Settings page

The app SHALL provide a `/settings` page where users can view and modify all preference fields.

#### Scenario: Settings page renders all sections

- **WHEN** the user navigates to `/settings`
- **THEN** the page SHALL display sections for: Display (theme, language), Units (unit system, distance, volume), Currency, Vehicles, and Data
- **AND** all section labels, button labels, and descriptive text SHALL use the active locale

#### Scenario: Language selector shows all supported locales

- **WHEN** the settings page renders the language control
- **THEN** it SHALL display chip-style segments for each supported locale: English, Deutsch
- **AND** the currently active locale SHALL be visually highlighted
- **AND** selecting a locale SHALL persist it and switch the active language

#### Scenario: Language change updates UI immediately

- **WHEN** the user selects a different language in the language selector
- **THEN** all visible text on the settings page SHALL update to the selected language without a page reload

#### Scenario: Theme control uses chip pattern

- **WHEN** the settings page renders the theme control
- **THEN** it SHALL display three chip-style segments: Light, Dark, System
- **AND** the currently active preference SHALL be visually highlighted
- **AND** selecting a theme SHALL apply and persist that preference

#### Scenario: Unit system selection with presets

- **WHEN** the user selects "Metric" as the unit system
- **THEN** distance unit SHALL be set to `km` and volume unit to `l`
- **AND** the individual unit selectors SHALL NOT be shown

#### Scenario: Unit system selection imperial

- **WHEN** the user selects "Imperial" as the unit system
- **THEN** distance unit SHALL be set to `mi` and volume unit to `gal`
- **AND** the individual unit selectors SHALL NOT be shown

#### Scenario: Custom unit system enables individual selectors

- **WHEN** the user selects "Custom" as the unit system
- **THEN** the distance unit and volume unit selectors SHALL become enabled
- **AND** the user SHALL be able to independently choose any valid distance and volume unit

#### Scenario: Currency selection

- **WHEN** the user selects a currency from the currency selector
- **THEN** the selected currency SHALL apply and persist

#### Scenario: Settings changes persist across page navigation

- **WHEN** the user changes a setting on `/settings` and navigates away
- **THEN** returning to `/settings` SHALL show the previously saved values

### Requirement: Settings page navigation

The settings page SHALL be accessible from the app's main navigation.

#### Scenario: Navigation link exists

- **WHEN** the app shell renders
- **THEN** a navigation link to `/settings` SHALL be visible
- **AND** it SHALL be marked active when the user is on the settings page or its sub-routes

### Requirement: Non-blocking preference initialization

At the start of each protected application session, current preferences SHALL begin loading automatically once. Protected routes SHALL render without waiting, using safe defaults until saved preferences become available.

#### Scenario: Preferences are loading

- **WHEN** a protected route renders while preference loading is pending
- **THEN** the route SHALL remain usable with defaults of `metric`, `km`, `l`, `EUR`, `system`, and `en`

#### Scenario: Saved preferences arrive

- **WHEN** preference loading succeeds
- **THEN** the protected application SHALL apply the saved preferences without a page reload

### Requirement: Data section on settings page

The settings page SHALL include a "Data" section for export and import controls, positioned after existing settings sections.

#### Scenario: Data section renders

- **WHEN** the user navigates to `/settings`
- **THEN** the page SHALL display a "Data" section containing an export button and an import area

### Requirement: Full export download button

The settings page SHALL provide a button to download a full data export.

#### Scenario: User clicks export button

- **WHEN** the user clicks the "Export data" button in the Data section
- **THEN** the browser SHALL download the JSON file from `GET /api/export`
- **AND** the file SHALL be saved with the filename from the `Content-Disposition` header

#### Scenario: Export button shows loading state

- **WHEN** the export download is in progress
- **THEN** the export button SHALL show a loading indicator
- **AND** the button SHALL be disabled until the download completes

### Requirement: Import upload flow

The settings page SHALL provide an import flow with file selection, preview, and confirmation.

#### Scenario: User selects import file

- **WHEN** the user clicks "Import data" and selects a JSON file
- **THEN** the UI SHALL send a `POST /api/import?preview=true` request with the file contents
- **AND** display the preview summary (number of vehicles and fill-ups to import)

#### Scenario: Preview shows replace mode warning

- **WHEN** the preview is displayed and mode is "replace"
- **THEN** the UI SHALL show a warning that existing data will be replaced

#### Scenario: User confirms import

- **WHEN** the user reviews the preview and clicks "Confirm import"
- **THEN** the UI SHALL send `POST /api/import` with the same file contents and selected mode
- **AND** show a success notification with the import summary
- **AND** the vehicle list SHALL immediately reflect the imported dataset
- **AND** the dashboard SHALL load fresh fill-ups and statistics when it next becomes active
- **AND** stale pre-import data SHALL NOT be displayed

#### Scenario: User cancels import

- **WHEN** the user reviews the preview and clicks "Cancel"
- **THEN** no import SHALL be performed and the preview SHALL be dismissed

#### Scenario: Import mode selection

- **WHEN** the import preview is shown
- **THEN** the UI SHALL allow the user to choose between "Replace" and "Merge" modes
- **AND** "Replace" SHALL be selected by default

#### Scenario: Import validation error

- **WHEN** the import preview or confirmation returns a validation error from the API
- **THEN** the UI SHALL display the error message to the user
- **AND** no data SHALL be modified

### Requirement: Per-vehicle export action

Each vehicle SHALL have an export action accessible from the vehicle list or detail view.

#### Scenario: User exports a single vehicle

- **WHEN** the user triggers the export action for a specific vehicle
- **THEN** the browser SHALL download the JSON file from `GET /api/vehicles/:id/export`

### Requirement: Settings exposes local logout when authentication is enabled
The settings page SHALL provide a translated logout action only when the authenticated app-info response reports that built-in authentication is enabled.

#### Scenario: Authentication-enabled settings page
- **WHEN** `/api/info` includes `auth_enabled: true`
- **THEN** the settings page SHALL display an Authentication section
- **AND** SHALL display a translated Sign out action
- **AND** activating the action SHALL send `POST /auth/logout` and follow its navigation response

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
The authenticated application-info response SHALL include `auth_enabled: true` only when built-in authentication is enabled.

#### Scenario: Disabled app-info compatibility
- **WHEN** built-in authentication is disabled
- **THEN** `/api/info` SHALL retain its existing response fields without adding `auth_enabled`

#### Scenario: Enabled app-info response
- **WHEN** built-in authentication is enabled and an authenticated client requests `/api/info`
- **THEN** the response SHALL include `auth_enabled: true`
