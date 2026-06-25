## MODIFIED Requirements

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
