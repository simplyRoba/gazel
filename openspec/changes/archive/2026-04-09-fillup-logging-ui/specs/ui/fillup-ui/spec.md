## ADDED Requirements

### Requirement: Fill-up API client types and functions

The API client SHALL export TypeScript types and functions for all fill-up CRUD operations.

#### Scenario: Fillup interface

- **WHEN** a `Fillup` object is used in the frontend
- **THEN** it SHALL have fields: `id` (number), `vehicle_id` (number), `date` (string), `odometer` (number), `fuel_amount` (number), `fuel_unit` (string), `cost` (number), `currency` (string), `is_full_tank` (boolean), `is_missed` (boolean), `station` (string | null), `notes` (string | null), `created_at` (string), `updated_at` (string)

#### Scenario: CreateFillup interface

- **WHEN** a `CreateFillup` object is sent to the API
- **THEN** it SHALL have `date` (string), `odometer` (number), `fuel_amount` (number), and `cost` (number) as required fields
- **AND** `is_full_tank` (boolean), `is_missed` (boolean), `station` (string), and `notes` (string) as optional fields
- **AND** it SHALL NOT include `fuel_unit` or `currency` (auto-populated by backend)

#### Scenario: UpdateFillup interface

- **WHEN** an `UpdateFillup` object is sent to the API
- **THEN** it SHALL have the same shape as `CreateFillup`

#### Scenario: Fetch fill-ups for a vehicle

- **WHEN** `fetchFillups(vehicleId)` is called
- **THEN** it SHALL send `GET /api/vehicles/{vehicleId}/fillups` and return `Fillup[]`

#### Scenario: Fetch single fill-up

- **WHEN** `fetchFillup(vehicleId, fillupId)` is called
- **THEN** it SHALL send `GET /api/vehicles/{vehicleId}/fillups/{fillupId}` and return `Fillup`

#### Scenario: Create fill-up

- **WHEN** `createFillup(vehicleId, data)` is called
- **THEN** it SHALL send `POST /api/vehicles/{vehicleId}/fillups` with the data and return the created `Fillup`

#### Scenario: Update fill-up

- **WHEN** `updateFillup(vehicleId, fillupId, data)` is called
- **THEN** it SHALL send `PUT /api/vehicles/{vehicleId}/fillups/{fillupId}` with the data and return the updated `Fillup`

#### Scenario: Delete fill-up

- **WHEN** `deleteFillup(vehicleId, fillupId)` is called
- **THEN** it SHALL send `DELETE /api/vehicles/{vehicleId}/fillups/{fillupId}` and return `void`

### Requirement: Fill-up store state

The fill-up store SHALL maintain a reactive cache of fill-ups keyed by vehicle ID, a loading flag, an error state, and the active vehicle ID.

#### Scenario: Initial state

- **WHEN** the store is first accessed
- **THEN** the fill-up cache SHALL be empty
- **AND** loading SHALL be `false`
- **AND** error SHALL be `null`
- **AND** active vehicle ID SHALL be `null`

#### Scenario: State accessors

- **WHEN** store state is read from a component
- **THEN** it SHALL be accessed via exported getter functions: `getFillups()`, `getLoading()`, `getError()`, `getActiveVehicleId()`

### Requirement: Fill-up store load action

The store SHALL provide a `loadFillups(vehicleId)` action that fetches fill-ups for a vehicle and caches them.

#### Scenario: Successful load

- **WHEN** `loadFillups(vehicleId)` is called and the API returns fill-ups
- **THEN** the fill-ups for that vehicle SHALL be stored in the cache
- **AND** error SHALL be `null`

#### Scenario: Load failure

- **WHEN** `loadFillups(vehicleId)` is called and the API throws an error
- **THEN** the cache SHALL remain unchanged
- **AND** error SHALL be set to the error message

#### Scenario: Cached data

- **WHEN** `loadFillups(vehicleId)` is called for a vehicle already in the cache
- **THEN** the cache SHALL be refreshed with the latest data from the API

### Requirement: Fill-up store create action

The store SHALL provide a `createFillup(vehicleId, data)` action that creates a fill-up and prepends it to the cached list.

#### Scenario: Successful create

- **WHEN** `createFillup(vehicleId, data)` is called and the API succeeds
- **THEN** the new fill-up SHALL be added to the cached list for that vehicle in the correct sort position (by date descending)
- **AND** the function SHALL return the created fill-up

#### Scenario: Create failure

- **WHEN** `createFillup(vehicleId, data)` is called and the API throws an error
- **THEN** the cache SHALL remain unchanged
- **AND** error SHALL be set
- **AND** the function SHALL return `null`

### Requirement: Fill-up store update action

The store SHALL provide an `updateFillup(vehicleId, fillupId, data)` action that replaces the fill-up in the cache.

#### Scenario: Successful update

- **WHEN** `updateFillup(vehicleId, fillupId, data)` is called and the API succeeds
- **THEN** the matching fill-up in the cache SHALL be replaced with the updated version
- **AND** the function SHALL return the updated fill-up

#### Scenario: Update failure

- **WHEN** `updateFillup(vehicleId, fillupId, data)` is called and the API throws an error
- **THEN** the cache SHALL remain unchanged
- **AND** error SHALL be set
- **AND** the function SHALL return `null`

### Requirement: Fill-up store delete action

The store SHALL provide a `deleteFillup(vehicleId, fillupId)` action that removes the fill-up from the cache.

#### Scenario: Successful delete

- **WHEN** `deleteFillup(vehicleId, fillupId)` is called and the API succeeds
- **THEN** the fill-up SHALL be removed from the cached list
- **AND** the function SHALL return `true`

#### Scenario: Delete failure

- **WHEN** `deleteFillup(vehicleId, fillupId)` is called and the API throws an error
- **THEN** the cache SHALL remain unchanged
- **AND** error SHALL be set
- **AND** the function SHALL return `false`

### Requirement: Fill-up store active vehicle

The store SHALL provide a `setActiveVehicle(vehicleId)` action and track which vehicle's fill-ups are currently displayed.

#### Scenario: Set active vehicle

- **WHEN** `setActiveVehicle(vehicleId)` is called
- **THEN** the active vehicle ID SHALL be updated
- **AND** `loadFillups(vehicleId)` SHALL be triggered

### Requirement: Fill-up store error clearing

Every store action SHALL clear the previous error before making an API call.

#### Scenario: Error is cleared on new action

- **WHEN** any store action is called
- **THEN** the error state SHALL be set to `null` before the API call is made

### Requirement: Dashboard vehicle chip interaction

The vehicle chips on the dashboard SHALL be interactive, selecting a vehicle and loading its fill-ups.

#### Scenario: Chip click selects vehicle

- **WHEN** the user taps a vehicle chip
- **THEN** that chip SHALL become active (visually highlighted)
- **AND** the fill-ups for that vehicle SHALL be loaded and displayed

#### Scenario: First vehicle selected on load

- **WHEN** the dashboard loads and vehicles are available
- **THEN** the first vehicle SHALL be automatically selected
- **AND** its fill-ups SHALL be loaded

#### Scenario: Active chip visual state

- **WHEN** a vehicle chip is active
- **THEN** it SHALL use the accent-colored active style
- **AND** all other chips SHALL use the default inactive style

### Requirement: Dashboard fill-up card list

The dashboard SHALL display fill-up cards for the selected vehicle below the chip row.

#### Scenario: Fill-ups displayed as cards

- **WHEN** the selected vehicle has fill-ups
- **THEN** each fill-up SHALL be rendered as a card showing: date, odometer reading (formatted per settings), fuel amount (formatted per settings), cost (formatted per settings)
- **AND** cards SHALL be sorted by date descending (most recent first)

#### Scenario: Optional fields on cards

- **WHEN** a fill-up has a station value
- **THEN** the station name SHALL be displayed on the card
- **WHEN** a fill-up has `is_full_tank` set to `true`
- **THEN** a visual indicator (badge or label) SHALL show "Full tank"

#### Scenario: Loading state

- **WHEN** fill-ups are being fetched
- **THEN** a loading indicator (shimmer/skeleton cards) SHALL be displayed

#### Scenario: Empty state

- **WHEN** the selected vehicle has no fill-ups
- **THEN** an empty state SHALL be displayed with a message like "No fill-ups yet" and a CTA to add the first fill-up

#### Scenario: Add fill-up button on dashboard

- **WHEN** the selected vehicle has fill-ups or is in empty state
- **THEN** an "Add fill-up" button SHALL be visible
- **AND** tapping it SHALL open the fill-up form modal for the active vehicle

### Requirement: Fill-up form modal

The fill-up form SHALL open as a modal dialog for both creating and editing fill-ups.

#### Scenario: Create mode

- **WHEN** the form modal opens without an existing fill-up
- **THEN** the title SHALL be "Add fill-up" (or similar)
- **AND** the date field SHALL default to today's date
- **AND** `is_full_tank` SHALL default to `true`
- **AND** `is_missed` SHALL default to `false`
- **AND** the submit button SHALL say "Save" (or similar)

#### Scenario: Edit mode

- **WHEN** the form modal opens with an existing fill-up
- **THEN** all fields SHALL be pre-filled with the fill-up's current values
- **AND** the title SHALL be "Edit fill-up" (or similar)
- **AND** a delete button SHALL be available

#### Scenario: Tapping a fill-up card opens edit mode

- **WHEN** the user taps a fill-up card on the dashboard
- **THEN** the fill-up form modal SHALL open in edit mode with that fill-up's data

#### Scenario: Form fields

- **WHEN** the fill-up form is displayed
- **THEN** it SHALL contain: date input (required), odometer input (required, with unit label from settings), fuel amount input (required, with unit label from settings), cost input (required, with currency symbol from settings), station input (optional), notes input (optional), is_full_tank toggle (default ON), is_missed toggle (default OFF)
- **AND** `fuel_unit` and `currency` SHALL NOT be form fields

#### Scenario: Client-side validation

- **WHEN** the user submits the form with missing required fields
- **THEN** field-level error messages SHALL be displayed
- **AND** the form SHALL NOT submit to the API

#### Scenario: Successful create submission

- **WHEN** the user submits a valid create form
- **THEN** the fill-up SHALL be created via the store
- **AND** the modal SHALL close
- **AND** the new fill-up SHALL appear in the card list

#### Scenario: Successful edit submission

- **WHEN** the user submits a valid edit form
- **THEN** the fill-up SHALL be updated via the store
- **AND** the modal SHALL close
- **AND** the card SHALL reflect the updated values

#### Scenario: Modal close

- **WHEN** the user presses Escape, clicks the backdrop, or taps a Cancel button
- **THEN** the modal SHALL close without saving

### Requirement: Fill-up delete confirmation

Deleting a fill-up SHALL require confirmation via the existing ModalDialog component.

#### Scenario: Delete from edit modal

- **WHEN** the user clicks the delete button in the edit form modal
- **THEN** a confirmation dialog SHALL appear with a warning message

#### Scenario: Confirm delete

- **WHEN** the user confirms the delete action
- **THEN** the fill-up SHALL be deleted via the store
- **AND** both the confirmation dialog and the edit modal SHALL close
- **AND** the card SHALL be removed from the list

#### Scenario: Cancel delete

- **WHEN** the user cancels the delete confirmation
- **THEN** the fill-up SHALL NOT be deleted
- **AND** the user SHALL return to the edit modal

### Requirement: Global CTA wiring

The CTA button in the app layout navigation SHALL open the fill-up form modal.

#### Scenario: CTA with one vehicle

- **WHEN** the user taps the CTA button
- **AND** exactly one vehicle exists
- **THEN** the fill-up form modal SHALL open immediately for that vehicle

#### Scenario: CTA with multiple vehicles

- **WHEN** the user taps the CTA button
- **AND** more than one vehicle exists
- **THEN** a vehicle picker SHALL be shown first
- **AND** after selecting a vehicle, the fill-up form modal SHALL open for that vehicle

#### Scenario: CTA with no vehicles

- **WHEN** the user taps the CTA button
- **AND** no vehicles exist
- **THEN** the user SHALL be directed to add a vehicle first (navigate to vehicle create page or show a message)

### Requirement: Smart missed fill-up prompt

The fill-up form SHALL detect suspiciously large odometer gaps and suggest the `is_missed` flag.

#### Scenario: Large odometer gap detected

- **WHEN** the user enters an odometer value in create mode
- **AND** the vehicle has at least 2 previous fill-ups
- **AND** the gap between the entered value and the last recorded odometer exceeds 1.75x the vehicle's average odometer gap
- **THEN** an inline prompt SHALL appear below the odometer field suggesting: "That's a larger gap than usual. Did you miss a fill-up?"
- **AND** the prompt SHALL offer a quick action to toggle `is_missed` to ON

#### Scenario: Normal odometer gap

- **WHEN** the user enters an odometer value
- **AND** the gap is within normal range (not exceeding 1.75x average)
- **THEN** no prompt SHALL be displayed

#### Scenario: Insufficient history

- **WHEN** the vehicle has fewer than 2 fill-ups
- **THEN** the smart prompt SHALL NOT be evaluated (not enough data for an average)
