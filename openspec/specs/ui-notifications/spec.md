## Purpose

Defines toast notification presentation, actions, positioning, visibility limits, dismissal, and semantic styling.

## Requirements

### Requirement: Toast presentation and actions

The application SHALL present notifications as dismissible toasts containing a message and optional action.

#### Scenario: Notification appears
- **WHEN** application behavior raises a notification
- **THEN** a toast SHALL display its message

#### Scenario: Optional action
- **WHEN** a notification includes an action
- **THEN** the action SHALL be available from the toast

#### Scenario: Manual dismissal
- **WHEN** the user closes a toast
- **THEN** that toast SHALL no longer be displayed

### Requirement: Responsive toast positioning

Toasts SHALL remain anchored to the viewport at a responsive location.

#### Scenario: Desktop positioning
- **WHEN** the viewport is wider than 768px
- **THEN** toasts SHALL appear at the bottom-right

#### Scenario: Mobile positioning
- **WHEN** the viewport is 768px or less
- **THEN** toasts SHALL appear at the top below the device safe area

#### Scenario: Maximum visible notifications
- **WHEN** more than three notifications are active
- **THEN** only the three most recent SHALL be visible

### Requirement: Automatic dismissal

Success and informational toasts SHALL dismiss automatically, while error toasts SHALL remain until closed by the user.

#### Scenario: Success or informational notification
- **WHEN** a success or informational toast appears
- **THEN** it SHALL dismiss automatically after 3500ms

#### Scenario: Error notification
- **WHEN** an error toast appears
- **THEN** it SHALL remain visible until the user closes it

#### Scenario: Hover pauses dismissal
- **WHEN** the user hovers over an automatically dismissing toast
- **THEN** its remaining dismissal time SHALL pause
- **AND** resume when the pointer leaves

### Requirement: Semantic toast styling

Success, error, and informational toasts SHALL be visually distinguishable.

#### Scenario: Variant appearance
- **WHEN** a toast is displayed
- **THEN** its left-border accent SHALL visually communicate whether it represents success, an error, or information
