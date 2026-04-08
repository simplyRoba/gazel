## ADDED Requirements

### Requirement: Confirm mode dialog

The ModalDialog SHALL display a confirmation dialog with Cancel and Confirm buttons.

#### Scenario: Confirm dialog opens
- **WHEN** `open` is set to `true` with `mode="confirm"`
- **THEN** a modal dialog SHALL appear with the title, message, Cancel button, and Confirm button

#### Scenario: Confirm callback
- **WHEN** the user clicks the Confirm button
- **THEN** the `onconfirm` callback SHALL be called

#### Scenario: Cancel callback
- **WHEN** the user clicks Cancel, presses Escape, or clicks the backdrop
- **THEN** the `oncancel` callback SHALL be called

### Requirement: Alert mode dialog

The ModalDialog SHALL display an alert dialog with a single OK button.

#### Scenario: Alert dialog
- **WHEN** `open` is set to `true` with `mode="alert"`
- **THEN** a modal dialog SHALL appear with the title, message, and a single OK button

#### Scenario: Alert close
- **WHEN** the user clicks OK or presses Escape
- **THEN** the `onclose` callback SHALL be called

### Requirement: Danger variant

The ModalDialog SHALL support a danger variant with an error-colored confirm button.

#### Scenario: Danger styling
- **WHEN** `variant="danger"` is set
- **THEN** the confirm/OK button SHALL use the error color (`--color-error`)

### Requirement: Native dialog element

The ModalDialog SHALL use the native `<dialog>` element with `showModal()` for proper focus trapping, backdrop, and accessibility.

#### Scenario: Focus management
- **WHEN** the dialog opens
- **THEN** focus SHALL be trapped within the dialog
- **AND** Escape key SHALL trigger cancel/close
