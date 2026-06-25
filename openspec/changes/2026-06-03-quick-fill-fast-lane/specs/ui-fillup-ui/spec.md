## ADDED Requirements

### Requirement: Numeric input guarding and normalization

Numeric fill-up inputs (odometer, fuel amount, price per unit, cost) SHALL guard against invalid characters during typing and normalize their displayed value on blur, using the user's locale. (Parsing of `.`/`,` decimals is specified under unit-formatting `parseDecimal`.)

#### Scenario: Keypress guarding

- **WHEN** a user types into a numeric fill-up input
- **THEN** only characters that can form a valid number SHALL be accepted (digits, a single decimal separator, and a leading sign where applicable)
- **AND** other characters SHALL be rejected so the field can never hold non-numeric garbage

#### Scenario: On-blur normalization

- **WHEN** a numeric fill-up input loses focus with a valid value
- **THEN** the displayed value SHALL be normalized to the user's locale formatting

## MODIFIED Requirements

### Requirement: Fill-up form modal

The fill-up form SHALL open as a modal dialog. Creating a fill-up SHALL use the Quick Fill fast-lane surface; editing a fill-up SHALL use the detailed form. The total/trip odometer mode toggle SHALL be available in both.

#### Scenario: Create mode uses Quick Fill

- **WHEN** the modal opens without an existing fill-up
- **THEN** it SHALL present the Quick Fill screen (large numeric inputs, fuel/price/total auto-calc, live efficiency preview, collapsible "More details")
- **AND** the date SHALL default to today's date
- **AND** `is_full_tank` SHALL default to `true`
- **AND** `is_missed` SHALL default to `false`
- **AND** the primary action SHALL save the fill-up

#### Scenario: Edit mode uses the detailed form

- **WHEN** the modal opens with an existing fill-up
- **THEN** it SHALL present the detailed form with all fields pre-filled with the fill-up's current values
- **AND** the title SHALL indicate edit mode
- **AND** a delete button SHALL be available

#### Scenario: Tapping a fill-up card opens edit mode

- **WHEN** the user taps a fill-up card on the dashboard
- **THEN** the fill-up form modal SHALL open in edit mode with that fill-up's data

#### Scenario: Detailed form fields

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
