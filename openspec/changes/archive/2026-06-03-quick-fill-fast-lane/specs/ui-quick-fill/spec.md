## ADDED Requirements

### Requirement: Quick Fill fast-lane create screen

The application SHALL provide a Quick Fill screen optimized for fast, mobile, one-handed fill-up creation. It SHALL be a single screen (not a multi-step wizard) presenting the essential receipt values prominently with optional details collapsed.

#### Scenario: Quick Fill is the default create surface

- **WHEN** the user triggers "Add fill-up" (from the dashboard add button or the global CTA) for a vehicle
- **THEN** the Quick Fill screen SHALL open for that vehicle
- **AND** it SHALL show large numeric inputs for fuel amount, price per unit, total cost, and odometer
- **AND** the date SHALL default to today without requiring input

#### Scenario: Odometer prefilled from last reading

- **WHEN** Quick Fill opens for a vehicle that has at least one prior fill-up
- **THEN** the odometer input SHALL be prefilled with the most recent reading (in the active total/trip mode)

#### Scenario: Single primary action

- **WHEN** Quick Fill is displayed
- **THEN** there SHALL be a single primary Save action
- **AND** the user SHALL be able to save a typical fill-up without opening the "More details" section

### Requirement: Fuel / price / total auto-calc

Quick Fill SHALL relate fuel amount, price per unit, and total cost by `total = fuel_amount * price_per_unit`, computing the missing value from the other two.

#### Scenario: Compute total from fuel and price

- **WHEN** the user has entered a valid fuel amount and a valid price per unit
- **AND** the total cost field has not been edited more recently than those two
- **THEN** the total cost SHALL be computed as `fuel_amount * price_per_unit`

#### Scenario: Compute price from fuel and total

- **WHEN** the user has entered a valid fuel amount (> 0) and a valid total cost
- **AND** the price field has not been edited more recently than those two
- **THEN** the price per unit SHALL be computed as `total / fuel_amount`

#### Scenario: Compute fuel from price and total

- **WHEN** the user has entered a valid price per unit (> 0) and a valid total cost
- **AND** the fuel field has not been edited more recently than those two
- **THEN** the fuel amount SHALL be computed as `total / price_per_unit`

#### Scenario: User-edited field is authoritative

- **WHEN** the user edits one of the three values
- **THEN** that field and the other most-recently-edited field SHALL be treated as authoritative
- **AND** only the remaining (least-recently-edited) field SHALL be recomputed
- **AND** the field currently being edited SHALL never be overwritten by auto-calc

#### Scenario: Division guard

- **WHEN** auto-calc would require dividing by a fuel amount or price that is zero, empty, or invalid
- **THEN** no value SHALL be computed
- **AND** typing SHALL not be blocked

#### Scenario: Price per unit is UI-only

- **WHEN** Quick Fill submits a fill-up
- **THEN** the request SHALL send `fuel_amount` and `cost` (total) only
- **AND** the price per unit SHALL NOT be sent to the API

### Requirement: Live efficiency preview

Quick Fill SHALL display a live preview of the computed efficiency while the user enters values, using the user's configured units.

#### Scenario: Preview shown with sufficient data

- **WHEN** the user has entered an odometer value and a fuel amount
- **AND** the vehicle has a prior full-tank reading enabling a valid segment with positive distance
- **THEN** Quick Fill SHALL display the computed efficiency formatted with the user's distance and volume units (e.g. `L/100km` or `mpg`)
- **AND** the preview SHALL update as the inputs change

#### Scenario: Preview hidden with insufficient data

- **WHEN** there is no prior reading enabling a valid segment, or the distance is not positive, or required inputs are missing
- **THEN** no efficiency preview SHALL be displayed

### Requirement: More details expander

Quick Fill SHALL collapse less-common fields behind a single "More details" expander, defaulting to collapsed.

#### Scenario: Default collapsed

- **WHEN** Quick Fill opens
- **THEN** the "More details" section SHALL be collapsed
- **AND** date, station, notes, the full-tank toggle, and the missed-fill-up toggle SHALL be hidden until expanded

#### Scenario: Expanding reveals detail fields

- **WHEN** the user opens "More details"
- **THEN** the date, station, notes, full-tank toggle, and missed-fill-up toggle SHALL become visible and editable
- **AND** any values already implied (e.g. date defaulting to today, full-tank defaulting to ON) SHALL be preserved

### Requirement: Quick Fill validation and smart prompt reuse

Quick Fill SHALL apply the same client-side validation rules and smart missed-fill-up detection used by the detailed fill-up form.

#### Scenario: Validation on save

- **WHEN** the user attempts to save with a missing or invalid required value (date, odometer, fuel amount, or total cost)
- **THEN** a field-level error SHALL be shown
- **AND** the fill-up SHALL NOT be submitted to the API

#### Scenario: Odometer minimum constraint

- **WHEN** the resolved odometer is less than the vehicle's most recent recorded odometer
- **THEN** a validation error SHALL be shown and the fill-up SHALL NOT be submitted

#### Scenario: Smart missed-fill-up prompt

- **WHEN** the entered odometer produces a gap exceeding 1.75x the vehicle's average gap and the vehicle has at least 2 prior fill-ups
- **THEN** Quick Fill SHALL show the missed-fill-up prompt with a quick action to set `is_missed` ON

#### Scenario: Successful save

- **WHEN** the user saves a valid Quick Fill entry
- **THEN** the fill-up SHALL be created via the fill-up store
- **AND** the screen SHALL close
- **AND** the new fill-up SHALL appear in the dashboard card list
