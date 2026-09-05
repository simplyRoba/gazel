## Purpose

Fill-up logging UI — vehicle chip selection on the dashboard, fill-up card list, form modal for create/edit, delete confirmation, global CTA wiring, and smart missed fill-up detection.

## Requirements

### Requirement: Fill-up API client types and functions

The API client SHALL export TypeScript types and functions for paginated fill-up retrieval and all fill-up CRUD operations.

#### Scenario: Fillup interface
- **WHEN** a `Fillup` object is used
- **THEN** it SHALL retain the existing fill-up fields and types

#### Scenario: FillupPage interface
- **WHEN** a fill-up list response is used
- **THEN** `FillupPage` SHALL contain `items: Fillup[]` and `next_cursor: string | null`

#### Scenario: CreateFillup and UpdateFillup interfaces
- **WHEN** create or update data is sent
- **THEN** it SHALL retain the existing required and optional fields
- **AND** it SHALL NOT include backend-populated `fuel_unit` or `currency`

#### Scenario: Fetch first page
- **WHEN** `fetchFillups(vehicleId)` is called without a cursor
- **THEN** it SHALL request the vehicle's first fill-up page and return `FillupPage`

#### Scenario: Fetch next page
- **WHEN** `fetchFillups(vehicleId, cursor)` is called with a server cursor
- **THEN** it SHALL pass the cursor unchanged and return the next `FillupPage`

#### Scenario: Fetch single fill-up
- **WHEN** `fetchFillup(vehicleId, fillupId)` is called
- **THEN** it SHALL return the requested `Fillup` through the existing endpoint

#### Scenario: Create, update, and delete fill-up
- **WHEN** an existing fill-up mutation client function is called
- **THEN** it SHALL retain its existing endpoint, payload, and return behavior

### Requirement: Incremental fill-up history state

The frontend SHALL maintain an independent paginated fill-up chain for each vehicle.

#### Scenario: Initial load
- **WHEN** a vehicle becomes active or is refreshed
- **THEN** its first page SHALL replace its loaded items and cursor
- **AND** a new request generation SHALL begin

#### Scenario: Continue loading
- **WHEN** a next cursor exists and continuation is requested
- **THEN** unseen items SHALL be appended in server order
- **AND** the returned cursor SHALL replace the previous cursor

#### Scenario: Guard continuation
- **WHEN** continuation is already active, paused after failure, or exhausted
- **THEN** another automatic continuation request SHALL NOT be sent

#### Scenario: Ignore stale response
- **WHEN** a response belongs to an earlier request generation
- **THEN** it SHALL NOT modify cached state

#### Scenario: Continuation failure and retry
- **WHEN** continuation fails
- **THEN** loaded items and the current cursor SHALL remain unchanged
- **AND** explicit retry SHALL request that cursor again

#### Scenario: Mutation refresh
- **WHEN** create, update, or delete succeeds
- **THEN** existing local cache behavior SHALL occur
- **AND** a fresh first-page generation SHALL replace stale pagination state

#### Scenario: Form helpers
- **WHEN** form assistance inspects fill-up history
- **THEN** it SHALL use currently loaded recent entries
- **AND** it SHALL NOT trigger continuation

### Requirement: Endless fill-up scrolling

The dashboard SHALL request older fill-up pages as the user approaches the end of loaded cards.

#### Scenario: Sentinel reached
- **WHEN** the sentinel approaches the active scroll viewport and a next cursor exists
- **THEN** continuation SHALL be requested once

#### Scenario: Loading more
- **WHEN** continuation is active
- **THEN** loaded cards SHALL remain visible
- **AND** a continuation loading indicator SHALL be shown

#### Scenario: Failed continuation
- **WHEN** continuation fails
- **THEN** automatic loading SHALL pause
- **AND** a retry action SHALL be shown

#### Scenario: Exhausted history
- **WHEN** no next cursor remains
- **THEN** the sentinel SHALL be removed
- **AND** scrolling SHALL trigger no further fill-up requests

#### Scenario: Observer lifecycle
- **WHEN** the active vehicle, scroll root, or component lifecycle changes
- **THEN** the old observer SHALL be disconnected before a new observer is created

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
- **THEN** each fill-up SHALL be rendered as a card showing: date, absolute odometer reading, fuel amount, and cost
- **AND** the absolute odometer SHALL use the selected locale's number formatting without a unit suffix
- **AND** the adjacent odometer difference SHALL use `formatDistance` with the selected locale and distance unit
- **AND** date, fuel amount, and cost SHALL be formatted with the corresponding locale, unit, and currency settings
- **AND** cards SHALL be sorted by date descending (most recent first)

#### Scenario: Optional fields on cards

- **WHEN** a fill-up has a station value
- **THEN** the station name SHALL be displayed on the card
- **WHEN** a fill-up has `is_full_tank` set to `true`
- **THEN** a visual indicator (badge or label) SHALL show "Full tank"

#### Scenario: Efficiency badge on fill-up card

- **WHEN** a fill-up is a full-tank fill that terminates a valid segment (matched via `end_date` and `end_odometer` from segment history)
- **THEN** the card SHALL display an efficiency badge showing the segment's efficiency value formatted with `formatEfficiency` using the user's distance_unit and volume_unit
- **AND** the badge SHALL be visually distinct (e.g., accent-colored)

#### Scenario: No efficiency data for fill-up

- **WHEN** a fill-up does not terminate a valid segment (partial tank, first fill-up, or segment is invalid)
- **THEN** no efficiency badge SHALL be displayed on that card

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

### Requirement: Numeric input guarding and normalization

Numeric fill-up inputs (odometer, fuel amount, price per unit, cost) SHALL guard keyboard entry against invalid characters while allowing pasted or dropped formatted values, then normalize parseable content on blur using the user's locale. (Parsing of `.`/`,` decimals is specified under unit-formatting `parseDecimal`.)

#### Scenario: Keyboard-entry guarding

- **WHEN** a user types into a numeric fill-up input
- **THEN** only characters that can form a valid number SHALL be accepted (digits, a single decimal separator, and a leading sign where applicable)
- **AND** other typed characters SHALL be rejected

#### Scenario: Pasted or dropped values

- **WHEN** content is pasted or dropped into a numeric input
- **THEN** it SHALL be accepted so values containing grouping separators, currency symbols, or unit labels can be parsed
- **AND** parseable content SHALL be normalized on blur
- **AND** unparseable content SHALL remain available for correction and SHALL be rejected by form validation

#### Scenario: On-blur normalization

- **WHEN** a numeric fill-up input loses focus with a valid value
- **THEN** the displayed value SHALL be normalized to the user's locale formatting

### Requirement: Fill-up form modal

The fill-up form SHALL open as a modal dialog. Creating a fill-up SHALL use the Quick Fill fast-lane surface; editing a fill-up SHALL use the detailed form. Trip-odometer entry SHALL be available only when creating a fill-up with an existing odometer baseline.

#### Scenario: Create mode

- **WHEN** the modal opens without an existing fill-up
- **THEN** it SHALL present the Quick Fill screen (large numeric inputs, fuel/price/total auto-calc, live efficiency preview, collapsible "More details")
- **AND** the date SHALL default to today's date
- **AND** `is_full_tank` SHALL default to `true`
- **AND** `is_missed` SHALL default to `false`
- **AND** the primary action SHALL save the fill-up

#### Scenario: Trip-odometer mode available

- **WHEN** the modal opens to create a fill-up and the vehicle has at least one prior fill-up
- **THEN** the total/trip odometer mode toggle SHALL be available
- **AND** trip mode SHALL add the entered distance to the previous absolute odometer

#### Scenario: Total-odometer mode required

- **WHEN** the modal opens for a vehicle without prior fill-ups or opens in edit mode
- **THEN** the total/trip toggle SHALL NOT be displayed
- **AND** the odometer field SHALL use total-odometer mode

#### Scenario: Edit mode

- **WHEN** the modal opens with an existing fill-up
- **THEN** it SHALL present the detailed form with all fields pre-filled with the fill-up's current values
- **AND** the title SHALL indicate edit mode
- **AND** a delete button SHALL be available

#### Scenario: Tapping a fill-up card opens edit mode

- **WHEN** the user taps a fill-up card on the dashboard
- **THEN** the fill-up form modal SHALL open in edit mode with that fill-up's data

#### Scenario: Form fields

- **WHEN** the detailed fill-up form is displayed (edit mode, or the expanded "More details" section in create mode)
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

The CTA button in the app layout navigation SHALL open the fill-up create surface (Quick Fill).

#### Scenario: CTA with one vehicle

- **WHEN** the user taps the CTA button
- **AND** exactly one vehicle exists
- **THEN** the Quick Fill create surface SHALL open immediately for that vehicle

#### Scenario: CTA with multiple vehicles

- **WHEN** the user taps the CTA button
- **AND** more than one vehicle exists
- **THEN** a vehicle picker SHALL be shown first
- **AND** after selecting a vehicle, the Quick Fill create surface SHALL open for that vehicle

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
