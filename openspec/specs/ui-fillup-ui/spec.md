## Purpose

Defines dashboard fill-up history, vehicle selection, fill-up cards, create/edit/delete interactions, global CTA behavior, and missed-fill-up assistance.

## Requirements

### Requirement: Incremental fill-up history

The dashboard SHALL maintain an independent chronological fill-up history for each vehicle, loading the newest page first and older pages as needed.

#### Scenario: Initial load or refresh
- **WHEN** a vehicle becomes active or its history is refreshed
- **THEN** its newest page SHALL replace the displayed history
- **AND** later responses from an earlier selection or refresh SHALL NOT alter the current history

#### Scenario: Continue loading
- **WHEN** older history is available and continuation is requested
- **THEN** unseen fill-ups SHALL be appended in server order
- **AND** future continuation SHALL begin at the next-page position returned by the server
- **AND** duplicate fill-ups SHALL NOT be displayed

#### Scenario: Duplicate continuation is prevented
- **WHEN** older history is already loading, paused after failure, or exhausted
- **THEN** another automatic continuation request SHALL NOT be sent

#### Scenario: Continuation failure and retry
- **WHEN** loading older history fails
- **THEN** already displayed fill-ups SHALL remain unchanged
- **AND** automatic loading SHALL pause
- **AND** an explicit retry action SHALL continue from the same position

#### Scenario: History exhausted
- **WHEN** no older fill-ups remain
- **THEN** scrolling SHALL trigger no further fill-up requests

#### Scenario: Form assistance uses recent history
- **WHEN** the form uses fill-up history for previews or suggestions
- **THEN** it SHALL use currently loaded recent entries
- **AND** SHALL NOT load older pages solely for form assistance

### Requirement: Endless fill-up scrolling

The dashboard SHALL automatically request older fill-ups as the user approaches the end of the displayed cards.

#### Scenario: User approaches the end
- **WHEN** the user approaches the end of loaded cards and older history is available
- **THEN** one continuation request SHALL begin

#### Scenario: Older history is loading
- **WHEN** a continuation request is active
- **THEN** loaded cards SHALL remain visible
- **AND** a continuation loading indicator SHALL be shown

#### Scenario: Older history fails to load
- **WHEN** a continuation request fails
- **THEN** a retry action SHALL be shown

### Requirement: Fill-up loading and mutation outcomes

Fill-up operations SHALL update the visible history on success, retain previously displayed data on failure, and provide clear loading and error feedback.

#### Scenario: Initial history load fails
- **WHEN** loading a vehicle's newest fill-ups fails
- **THEN** previously displayed fill-ups SHALL remain unchanged
- **AND** a user-facing error SHALL be shown

#### Scenario: New operation begins after failure
- **WHEN** a new fill-up load or mutation begins after a failure
- **THEN** the previous general error SHALL be cleared
- **AND** the applicable loading state SHALL be shown

#### Scenario: Fill-up creation succeeds
- **WHEN** a fill-up is created successfully
- **THEN** the visible history SHALL refresh from its newest page
- **AND** the new fill-up SHALL appear in descending date order

#### Scenario: Fill-up update succeeds
- **WHEN** a fill-up is updated successfully
- **THEN** the visible history SHALL refresh from its newest page
- **AND** the card SHALL reflect the updated values in descending date order

#### Scenario: Fill-up deletion succeeds
- **WHEN** a fill-up is deleted successfully
- **THEN** the visible history SHALL refresh from its newest page
- **AND** the deleted fill-up SHALL no longer appear

#### Scenario: Fill-up mutation fails
- **WHEN** creating, updating, or deleting a fill-up fails
- **THEN** previously displayed fill-ups SHALL remain unchanged
- **AND** a user-facing error SHALL be shown

### Requirement: Dashboard vehicle chip interaction

Vehicle chips SHALL select which vehicle's fill-ups are displayed.

#### Scenario: Chip click selects vehicle
- **WHEN** the user taps a vehicle chip
- **THEN** that chip SHALL become visually active
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
- **THEN** each card SHALL show the date, absolute odometer reading, fuel amount, and cost
- **AND** the absolute odometer SHALL use the selected locale's number formatting without a unit suffix
- **AND** the adjacent odometer difference SHALL use the selected locale and distance unit
- **AND** date, fuel amount, and cost SHALL use the corresponding locale, unit, and currency settings
- **AND** cards SHALL be sorted by date descending

#### Scenario: Optional fields on cards
- **WHEN** a fill-up has a station value
- **THEN** the station name SHALL be displayed
- **WHEN** a fill-up is marked as a full tank
- **THEN** a visual indicator SHALL show `Full tank`

#### Scenario: Efficiency badge on fill-up card
- **WHEN** a full-tank fill-up terminates a valid segment matched by end date and odometer
- **THEN** its card SHALL display the segment's efficiency using the user's distance and volume units
- **AND** the badge SHALL be visually distinct

#### Scenario: No efficiency data for fill-up
- **WHEN** a fill-up does not terminate a valid segment
- **THEN** no efficiency badge SHALL be displayed on that card

#### Scenario: Loading state
- **WHEN** the newest fill-ups are being fetched
- **THEN** shimmer or skeleton cards SHALL be displayed

#### Scenario: Empty state
- **WHEN** the selected vehicle has no fill-ups
- **THEN** an empty state SHALL indicate that no fill-ups exist
- **AND** an action to add the first fill-up SHALL be available

#### Scenario: Add fill-up button on dashboard
- **WHEN** a vehicle is selected
- **THEN** an `Add fill-up` button SHALL be visible
- **AND** tapping it SHALL open fill-up creation for that vehicle

### Requirement: Numeric input guarding and normalization

Numeric fill-up inputs SHALL reject invalid keyboard entry while allowing pasted or dropped formatted values, then normalize parseable content on blur using the user's locale.

#### Scenario: Keyboard-entry guarding
- **WHEN** a user types into a numeric fill-up input
- **THEN** only characters that can form a valid number SHALL be accepted
- **AND** other typed characters SHALL be rejected

#### Scenario: Pasted or dropped values
- **WHEN** content is pasted or dropped into a numeric input
- **THEN** values containing grouping separators, currency symbols, or unit labels SHALL be accepted for parsing
- **AND** parseable content SHALL be normalized on blur
- **AND** unparseable content SHALL remain available for correction and be rejected by form validation

#### Scenario: On-blur normalization
- **WHEN** a numeric fill-up input loses focus with a valid value
- **THEN** the displayed value SHALL be normalized to the user's locale formatting

### Requirement: Fill-up form modal

Fill-up creation and editing SHALL open as modal dialogs. Creation SHALL use Quick Fill, editing SHALL use the detailed form, and trip-odometer entry SHALL be available only for creation with an existing odometer baseline.

#### Scenario: Create mode
- **WHEN** fill-up creation opens
- **THEN** it SHALL present large numeric inputs, automatically calculated fuel/price/total values, a live efficiency preview, and a collapsible `More details` section
- **AND** the date SHALL default to today
- **AND** Full tank SHALL default to on
- **AND** Missed fill-up SHALL default to off
- **AND** the primary action SHALL save the fill-up

#### Scenario: Trip-odometer mode available
- **WHEN** fill-up creation opens for a vehicle with at least one prior fill-up
- **THEN** the total/trip odometer toggle SHALL be available
- **AND** trip mode SHALL add the entered distance to the previous absolute odometer

#### Scenario: Total-odometer mode required
- **WHEN** creation opens for a vehicle without prior fill-ups or an existing fill-up is edited
- **THEN** the total/trip toggle SHALL NOT be displayed
- **AND** the odometer field SHALL use total-odometer mode

#### Scenario: Edit mode
- **WHEN** an existing fill-up is opened
- **THEN** the detailed form SHALL contain its current values
- **AND** the title SHALL indicate edit mode
- **AND** a delete action SHALL be available

#### Scenario: Tapping a fill-up card opens edit mode
- **WHEN** the user taps a fill-up card
- **THEN** that fill-up SHALL open for editing

#### Scenario: Detailed form fields
- **WHEN** an existing fill-up is edited or Quick Fill's `More details` section is expanded
- **THEN** the form SHALL contain required date, odometer, fuel amount, and cost inputs
- **AND** it SHALL contain optional station and notes inputs
- **AND** it SHALL contain Full tank and Missed fill-up toggles
- **AND** applicable unit and currency labels SHALL use current settings
- **AND** fuel unit and currency SHALL NOT be editable fields

#### Scenario: Client-side validation
- **WHEN** the user submits a form with missing required fields
- **THEN** field-level error messages SHALL be displayed
- **AND** no save request SHALL be sent

#### Scenario: Successful create submission
- **WHEN** the user submits valid creation data
- **THEN** the fill-up SHALL be saved
- **AND** the creation surface SHALL close
- **AND** the new fill-up SHALL appear in the card list

#### Scenario: Successful edit submission
- **WHEN** the user submits valid changes
- **THEN** the fill-up SHALL be updated
- **AND** the edit surface SHALL close
- **AND** the card SHALL reflect the updated values

#### Scenario: Form close
- **WHEN** the user presses Escape, clicks the backdrop, or taps Cancel
- **THEN** the form SHALL close without saving

### Requirement: Fill-up delete confirmation

Deleting a fill-up SHALL require explicit confirmation.

#### Scenario: Delete from edit form
- **WHEN** the user chooses delete while editing
- **THEN** a confirmation dialog SHALL appear with a warning message

#### Scenario: Confirm delete
- **WHEN** the user confirms deletion
- **THEN** the fill-up SHALL be deleted
- **AND** the confirmation and edit surfaces SHALL close
- **AND** the card SHALL be removed from the list

#### Scenario: Cancel delete
- **WHEN** the user cancels deletion
- **THEN** the fill-up SHALL NOT be deleted
- **AND** the user SHALL return to editing

### Requirement: Global fill-up CTA

The navigation CTA SHALL open Quick Fill creation.

#### Scenario: CTA with one vehicle
- **WHEN** the user taps the CTA and exactly one vehicle exists
- **THEN** Quick Fill SHALL open immediately for that vehicle

#### Scenario: CTA with multiple vehicles
- **WHEN** the user taps the CTA and multiple vehicles exist
- **THEN** a vehicle picker SHALL be shown first
- **AND** Quick Fill SHALL open for the selected vehicle

#### Scenario: CTA with no vehicles
- **WHEN** the user taps the CTA and no vehicles exist
- **THEN** the user SHALL be directed to add a vehicle first

### Requirement: Smart missed fill-up prompt

The fill-up form SHALL detect suspiciously large odometer gaps and suggest marking the fill-up as missed.

#### Scenario: Large odometer gap detected
- **WHEN** the user enters an odometer value during creation
- **AND** the vehicle has at least two previous fill-ups
- **AND** the gap exceeds 1.75 times the vehicle's average odometer gap
- **THEN** an inline prompt below the odometer field SHALL say `That's a larger gap than usual. Did you miss a fill-up?`
- **AND** the prompt SHALL offer an action that directly marks the fill-up as missed

#### Scenario: Normal odometer gap
- **WHEN** the entered gap does not exceed 1.75 times the average
- **THEN** no missed-fill-up prompt SHALL be displayed

#### Scenario: Insufficient history
- **WHEN** the vehicle has fewer than two previous fill-ups
- **THEN** the missed-fill-up prompt SHALL NOT be evaluated
