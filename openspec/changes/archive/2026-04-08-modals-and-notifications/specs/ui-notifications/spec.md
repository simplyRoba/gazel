## ADDED Requirements

### Requirement: Push notifications

The notification store SHALL allow pushing notifications with a message, variant, and optional action.

#### Scenario: Push a notification
- **WHEN** `pushNotification()` is called with a message and variant
- **THEN** a notification SHALL be added to the store with a generated ID

#### Scenario: Dismiss a notification
- **WHEN** `dismissNotification(id)` is called
- **THEN** the notification with that ID SHALL be removed from the store

### Requirement: Toast rendering

The ToastHost SHALL render visible notifications as fixed-position toasts.

#### Scenario: Desktop positioning
- **WHEN** the viewport is wider than 768px
- **THEN** toasts SHALL appear fixed at the bottom-right

#### Scenario: Mobile positioning
- **WHEN** the viewport is 768px or less
- **THEN** toasts SHALL appear fixed at the top, below the safe-area inset

#### Scenario: Max visible
- **WHEN** more than 3 notifications are in the store
- **THEN** only the 3 most recent SHALL be visible

### Requirement: Auto-dismiss behavior

Success and info toasts SHALL auto-dismiss; error toasts SHALL persist.

#### Scenario: Success auto-dismiss
- **WHEN** a success notification is pushed
- **THEN** it SHALL be automatically dismissed after 3500ms

#### Scenario: Error persists
- **WHEN** an error notification is pushed
- **THEN** it SHALL remain visible until the user clicks the close button

#### Scenario: Hover pauses dismiss
- **WHEN** the user hovers over an auto-dismissing toast
- **THEN** the dismiss timer SHALL pause and resume on mouse leave

### Requirement: Variant styling

Each toast SHALL be visually distinguished by its variant.

#### Scenario: Variant colors
- **WHEN** a toast is rendered
- **THEN** success SHALL use `--color-success`, error SHALL use `--color-error`, info SHALL use `--color-info` as the left border accent color
