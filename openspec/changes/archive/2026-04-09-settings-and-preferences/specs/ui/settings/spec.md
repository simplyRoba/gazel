## ADDED Requirements

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
- **THEN** the page SHALL display sections for: Display (theme, locale), Units (unit system, distance, volume), and Currency

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
