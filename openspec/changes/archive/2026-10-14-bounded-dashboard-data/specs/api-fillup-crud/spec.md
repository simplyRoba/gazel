# Spec Delta

## ADDED Requirements

### Requirement: Page-local efficiency metadata

Each fill-up list page SHALL include an `efficiency` metadata object containing `distance_unit`, `volume_unit`, and `values`. Each value SHALL identify a returned `fillup_id` and its nullable efficiency. Metadata SHALL cover only the current page. Individual fill-up objects, write payloads, ordering, page-size limits, and cursors SHALL remain unchanged.

#### Scenario: Bounded metadata
- **WHEN** a page contains 25 fill-ups from a much larger history
- **THEN** `efficiency.values` SHALL contain exactly one value per returned fill-up
- **AND** no value SHALL describe an entry outside that page
- **AND** no complete segment or fill-up history SHALL be included

#### Scenario: Segment begins outside the page
- **WHEN** a returned fill-up completes a valid tank-to-tank segment whose starting full tank is outside the requested page
- **THEN** its efficiency SHALL include the complete segment's eligible intervening fuel
- **AND** the client SHALL NOT need to fetch the starting full tank to display the badge

#### Scenario: Invalid or absent segment
- **WHEN** a returned fill-up does not end a valid positive-distance tank-to-tank segment
- **THEN** its efficiency SHALL be `null`
- **AND** partial fills and missed-segment endings SHALL NOT receive a valid efficiency badge

#### Scenario: Same-date fill-ups
- **WHEN** several fill-ups share a date or odometer value
- **THEN** metadata SHALL identify the actual segment-ending fill-up by ID using the existing date-then-ID chronology
- **AND** unrelated entries SHALL NOT inherit a badge through a shared date/odometer key

#### Scenario: Units and rounding
- **WHEN** the stored fuel units differ from current display units
- **THEN** each efficiency SHALL use the existing segment conversion and two-decimal rounding rules
- **AND** metadata SHALL identify the distance and volume units used

#### Scenario: Empty page
- **WHEN** no fill-ups are returned
- **THEN** `efficiency.values` SHALL be empty
- **AND** existing terminal cursor behavior SHALL remain unchanged
