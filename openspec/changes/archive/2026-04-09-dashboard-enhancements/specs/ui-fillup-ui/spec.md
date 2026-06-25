## MODIFIED Requirements

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
