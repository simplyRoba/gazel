## Purpose

Defines the global settings store, settings page UI, navigation, and app-level hydration for user preferences.

## Requirements

### Requirement: Global settings store

The app SHALL maintain a global settings store (`settings.svelte.ts`) using Svelte 5 runes that holds the current user preferences and exposes them reactively.

#### Scenario: Store initialization on app load

- **WHEN** the app loads and the root layout mounts
- **THEN** the settings store SHALL call `GET /api/settings`
- **AND** populate all preference fields from the server response
- **AND** expose the settings reactively via `getSettings()`

#### Scenario: Store exposes all preference fields

- **WHEN** the settings store is initialized
- **THEN** it SHALL expose: `unit_system`, `distance_unit`, `volume_unit`, `currency`, `color_mode`, `locale`

#### Scenario: Settings update propagates to server

- **WHEN** a setting is changed via the store's `updateSettings()` function
- **THEN** the store SHALL send a `PUT /api/settings` request with the changed fields
- **AND** update the local state optimistically
- **AND** revert the local state if the API call fails

#### Scenario: Settings fetch failure

- **WHEN** `GET /api/settings` fails during initialization
- **THEN** the store SHALL use sensible client-side defaults (`metric`, `km`, `l`, `USD`, `system`, `en`)
- **AND** the app SHALL remain functional

### Requirement: Settings page

The app SHALL provide a `/settings` page where users can view and modify all preference fields.

#### Scenario: Settings page renders all sections

- **WHEN** the user navigates to `/settings`
- **THEN** the page SHALL display sections for: Display (theme, language), Units (unit system, distance, volume), Currency, Vehicles, and Data
- **AND** all section labels, button labels, and descriptive text SHALL be rendered via `t()` translation calls

#### Scenario: Language selector shows all supported locales

- **WHEN** the settings page renders the language control
- **THEN** it SHALL display chip-style segments for each supported locale: English, Deutsch
- **AND** the currently active locale SHALL be visually highlighted
- **AND** clicking a locale chip SHALL call `updateSettingsStore({ locale })` to persist and switch the active language

#### Scenario: Language change updates UI immediately

- **WHEN** the user selects a different language in the language selector
- **THEN** all visible text on the settings page SHALL update to the selected language without a page reload

#### Scenario: Theme control uses chip pattern

- **WHEN** the settings page renders the theme control
- **THEN** it SHALL display three chip-style segments: Light, Dark, System
- **AND** the currently active preference SHALL be visually highlighted
- **AND** clicking a chip SHALL call `setTheme()` with the selected value

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
- **THEN** the settings store SHALL update the `currency` field
- **AND** a `PUT /api/settings` request SHALL be sent with the new currency

#### Scenario: Settings changes persist across page navigation

- **WHEN** the user changes a setting on `/settings` and navigates away
- **THEN** returning to `/settings` SHALL show the previously saved values

### Requirement: Settings page navigation

The settings page SHALL be accessible from the app's main navigation.

#### Scenario: Navigation link exists

- **WHEN** the app shell renders
- **THEN** a navigation link to `/settings` SHALL be visible
- **AND** it SHALL be marked active when the user is on the settings page or its sub-routes

### Requirement: Settings hydration on app init

The root layout SHALL fetch settings once on initial load and seed the global store before child routes render.

#### Scenario: Settings available before first route renders

- **WHEN** the app starts
- **THEN** the root layout SHALL call `initSettings()` during mount
- **AND** child components SHALL be able to read settings reactively from the store

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
- **AND** refresh the vehicle and fill-up stores

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
