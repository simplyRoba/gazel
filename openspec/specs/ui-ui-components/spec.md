## Purpose

Defines consistent page-width and empty-state presentation patterns.

## Requirements

### Requirement: Constrained page content

Pages SHALL support narrow, default, and wide content widths as defined by the application layout and SHALL center constrained content within the available page area.

#### Scenario: Default-width page
- **WHEN** a page does not select another width variant
- **THEN** its content SHALL use the default maximum width
- **AND** remain horizontally centered

#### Scenario: Narrow-width page
- **WHEN** a page selects the narrow width variant
- **THEN** its content SHALL use the narrow maximum width
- **AND** remain horizontally centered

#### Scenario: Wide-width page
- **WHEN** a page selects the wide width variant
- **THEN** its content SHALL use the wide maximum width
- **AND** remain horizontally centered

### Requirement: Empty-state presentation

When a list or page presents an empty state, it SHALL center an icon, heading, description, and optional action.

#### Scenario: Complete empty state
- **WHEN** an empty state includes an icon, heading, description, and action
- **THEN** the icon SHALL appear above the heading
- **AND** the heading SHALL appear prominent and semibold
- **AND** the description SHALL appear below it with visually secondary emphasis
- **AND** the action SHALL appear below the description

#### Scenario: Empty state without an action
- **WHEN** an empty state has no available action
- **THEN** the icon, heading, and description SHALL remain visible
- **AND** no empty action area SHALL be displayed

#### Scenario: Empty-state alignment
- **WHEN** an empty state is displayed
- **THEN** its content SHALL be vertically and horizontally centered within the available area
- **AND** the icon, heading, description, and optional action SHALL have consistent spacing
