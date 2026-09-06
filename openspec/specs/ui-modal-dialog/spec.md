## Purpose

Defines confirmation and alert modal behavior, destructive-action emphasis, dismissal, and accessible focus handling.

## Requirements

### Requirement: Confirmation modal

A confirmation modal SHALL display its title and message with Cancel and Confirm actions.

#### Scenario: Confirmation opens
- **WHEN** an interaction requests confirmation
- **THEN** a modal SHALL appear with the requested title and message
- **AND** Cancel and Confirm actions SHALL be available

#### Scenario: User confirms
- **WHEN** the user chooses Confirm
- **THEN** the requested action SHALL proceed

#### Scenario: User cancels
- **WHEN** the user chooses Cancel, presses Escape, or clicks the backdrop
- **THEN** the interaction SHALL be canceled without performing the requested action

### Requirement: Alert modal

An alert modal SHALL display its title and message with a single OK action.

#### Scenario: Alert opens
- **WHEN** an interaction presents an alert
- **THEN** a modal SHALL appear with the requested title and message
- **AND** an OK action SHALL be available

#### Scenario: Alert closes
- **WHEN** the user chooses OK or presses Escape
- **THEN** the alert SHALL close

### Requirement: Destructive-action emphasis

A modal confirming a destructive action SHALL visually distinguish its confirmation action as dangerous.

#### Scenario: Destructive confirmation
- **WHEN** a modal asks the user to confirm a destructive action
- **THEN** its confirmation action SHALL use the application's error styling

### Requirement: Accessible modal interaction

An open modal SHALL contain keyboard focus and prevent interaction with the underlying page until the modal is resolved.

#### Scenario: Focus management
- **WHEN** a modal opens
- **THEN** keyboard focus SHALL remain within the modal
- **AND** the underlying page SHALL not be interactive
- **AND** Escape SHALL trigger the modal's defined cancel or close behavior
