## ADDED Requirements

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
