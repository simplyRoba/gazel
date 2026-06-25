## ADDED Requirements

### Requirement: Vehicle form component

A shared `VehicleForm` component SHALL render a form with fields for name, make, model, year, fuel type, and notes, usable for both creating and editing vehicles.

#### Scenario: Empty form for create
- **WHEN** `VehicleForm` is rendered without an `initial` prop
- **THEN** all fields SHALL be empty except fuel type which SHALL default to "gasoline"

#### Scenario: Pre-filled form for edit
- **WHEN** `VehicleForm` is rendered with an `initial` vehicle
- **THEN** all fields SHALL be populated with the vehicle's current values

#### Scenario: Name validation
- **WHEN** the user submits the form with an empty or whitespace-only name
- **THEN** a validation error SHALL be shown inline
- **AND** the form SHALL NOT call the `onsave` callback

#### Scenario: Successful submission
- **WHEN** the user submits a valid form
- **THEN** the `onsave` callback SHALL be called with the form data as a `CreateVehicle` object

### Requirement: Create vehicle page

A page at `/settings/vehicles/new` SHALL allow users to create a new vehicle.

#### Scenario: Successful create and navigate
- **WHEN** the user fills out the form and saves
- **AND** the API returns successfully
- **THEN** the user SHALL be navigated back to `/settings`

#### Scenario: Create API error
- **WHEN** the user saves and the API returns an error
- **THEN** the error message SHALL be displayed on the page
- **AND** the user SHALL remain on the form page

### Requirement: Edit vehicle page

A page at `/settings/vehicles/[id]/edit` SHALL allow users to edit an existing vehicle.

#### Scenario: Load and display vehicle
- **WHEN** the edit page is loaded
- **THEN** the vehicle SHALL be fetched from the API by ID
- **AND** the form SHALL be pre-filled with the vehicle's data

#### Scenario: Successful edit and navigate
- **WHEN** the user edits the form and saves
- **AND** the API returns successfully
- **THEN** the user SHALL be navigated back to `/settings`

#### Scenario: Vehicle not found
- **WHEN** the edit page is loaded with an invalid ID
- **THEN** an error message SHALL be displayed

### Requirement: Settings vehicles section

The settings page SHALL display a "Vehicles" section listing all vehicles with edit and delete actions.

#### Scenario: Vehicles displayed
- **WHEN** the settings page loads and vehicles exist
- **THEN** each vehicle SHALL be displayed as a row with name, make/model/year, and action buttons

#### Scenario: No vehicles
- **WHEN** the settings page loads and no vehicles exist
- **THEN** an empty state message SHALL be displayed with an "Add vehicle" action

#### Scenario: Add vehicle button
- **WHEN** the user clicks "Add vehicle"
- **THEN** the user SHALL be navigated to `/settings/vehicles/new`

#### Scenario: Edit button
- **WHEN** the user clicks the edit button on a vehicle row
- **THEN** the user SHALL be navigated to `/settings/vehicles/[id]/edit`

### Requirement: Delete confirmation

Deleting a vehicle SHALL require an inline confirmation step.

#### Scenario: Delete initiation
- **WHEN** the user clicks the delete button on a vehicle row
- **THEN** the row SHALL transform to show a confirmation prompt with confirm and cancel buttons

#### Scenario: Delete confirmed
- **WHEN** the user confirms deletion
- **THEN** the vehicle SHALL be deleted via the store
- **AND** the vehicle SHALL be removed from the list

#### Scenario: Delete cancelled
- **WHEN** the user cancels deletion
- **THEN** the row SHALL revert to its normal display
- **AND** no API call SHALL be made

### Requirement: Dashboard empty state with vehicle link

When no vehicles exist, the dashboard SHALL show an empty state directing the user to add their first vehicle.

#### Scenario: No vehicles on dashboard
- **WHEN** the dashboard loads and the vehicle list is empty
- **THEN** an empty state SHALL be displayed with a button linking to `/settings/vehicles/new`
