## Purpose

Defines vehicle create/edit forms, settings-page vehicle management, dashboard empty-state navigation, deletion confirmation, and visible operation outcomes.

## Requirements

### Requirement: Vehicle form behavior

Vehicle creation and editing SHALL provide fields for name, make, model, year, fuel type, and notes.

#### Scenario: New vehicle form
- **WHEN** the user opens vehicle creation
- **THEN** all fields SHALL be empty except fuel type
- **AND** fuel type SHALL default to `gasoline`

#### Scenario: Edit vehicle form
- **WHEN** the user opens an existing vehicle for editing
- **THEN** all fields SHALL contain the vehicle's current values

#### Scenario: Name validation
- **WHEN** the user submits an empty or whitespace-only name
- **THEN** an inline validation error SHALL be shown
- **AND** no save request SHALL be sent

#### Scenario: Valid form
- **WHEN** the user submits valid vehicle data
- **THEN** the data SHALL be submitted for creation or update as appropriate

### Requirement: Create vehicle page

`/settings/vehicles/new` SHALL allow users to create a vehicle.

#### Scenario: Successful creation
- **WHEN** the user submits valid creation data and saving succeeds
- **THEN** the user SHALL return to `/settings`
- **AND** the created vehicle SHALL appear in the vehicle list

#### Scenario: Creation fails
- **WHEN** saving a new vehicle fails
- **THEN** the API error message SHALL be displayed
- **AND** the user SHALL remain on the creation page

### Requirement: Edit vehicle page

`/settings/vehicles/{id}/edit` SHALL allow users to edit an existing vehicle.

#### Scenario: Vehicle loads
- **WHEN** the user opens a valid vehicle edit URL
- **THEN** the latest data for the vehicle identified by the URL SHALL be loaded
- **AND** the form SHALL display that data

#### Scenario: Successful update
- **WHEN** the user submits valid changes and saving succeeds
- **THEN** the user SHALL return to `/settings`
- **AND** the vehicle list SHALL reflect the updated values

#### Scenario: Vehicle not found
- **WHEN** the user opens an edit URL with an unknown vehicle ID
- **THEN** an error message SHALL be displayed

### Requirement: Settings vehicle list

The settings page SHALL load and display the current vehicle list with edit and delete actions.

#### Scenario: Vehicle list loads
- **WHEN** loading or refreshing the vehicle list succeeds
- **THEN** the displayed list SHALL be replaced with the current vehicles

#### Scenario: Vehicles exist
- **WHEN** the vehicle list contains vehicles
- **THEN** each vehicle SHALL appear as a row with its name, make, model, year, and available actions

#### Scenario: No vehicles exist
- **WHEN** the vehicle list is empty
- **THEN** an empty state SHALL provide an `Add vehicle` action

#### Scenario: Add vehicle
- **WHEN** the user chooses `Add vehicle`
- **THEN** navigation SHALL open `/settings/vehicles/new`

#### Scenario: Edit vehicle
- **WHEN** the user chooses edit for a vehicle
- **THEN** navigation SHALL open `/settings/vehicles/{id}/edit` for that vehicle

#### Scenario: Vehicle list refresh fails
- **WHEN** refreshing the vehicle list fails
- **THEN** already displayed vehicles SHALL remain unchanged
- **AND** a notification SHALL display the API error message

### Requirement: Dashboard empty state links to vehicle creation

When no vehicles exist, the dashboard SHALL direct the user to create the first vehicle.

#### Scenario: Dashboard has no vehicles
- **WHEN** the dashboard loads with no vehicles
- **THEN** an empty state SHALL provide a link to `/settings/vehicles/new`

### Requirement: Vehicle deletion confirmation

Vehicle deletion SHALL require a destructive confirmation modal containing the vehicle name.

#### Scenario: Delete requested
- **WHEN** the user chooses delete for a vehicle
- **THEN** a destructive confirmation modal SHALL open
- **AND** its message SHALL identify the vehicle

#### Scenario: Delete confirmed
- **WHEN** the user confirms deletion and it succeeds
- **THEN** the modal SHALL close
- **AND** the vehicle SHALL be removed from the displayed list

#### Scenario: Deletion fails
- **WHEN** the user confirms deletion and it fails
- **THEN** the modal SHALL remain open
- **AND** the vehicle SHALL remain in the displayed list
- **AND** a notification SHALL display the API error message

#### Scenario: Delete canceled
- **WHEN** the user cancels deletion
- **THEN** the modal SHALL close
- **AND** no deletion request SHALL be sent

### Requirement: Vehicle operation errors

Failed vehicle loading, creation, update, or deletion SHALL preserve already displayed vehicle data and show a notification containing the API error message.

#### Scenario: Vehicle operation fails
- **WHEN** a vehicle operation fails
- **THEN** already displayed vehicle data SHALL remain unchanged
- **AND** an error-styled notification SHALL display the API error message
