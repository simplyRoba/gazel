## MODIFIED Requirements

### Requirement: Delete confirmation (modified)

Vehicle deletion SHALL use a ModalDialog instead of inline row confirmation.

#### Scenario: Delete triggers modal
- **WHEN** the user clicks the delete button on a vehicle row
- **THEN** a ModalDialog SHALL open with `mode="confirm"`, `variant="danger"`, and the vehicle name in the message

#### Scenario: Confirm deletes vehicle
- **WHEN** the user confirms deletion in the modal
- **THEN** the vehicle SHALL be deleted via the store
- **AND** the modal SHALL close

#### Scenario: Cancel closes modal
- **WHEN** the user cancels in the modal
- **THEN** the modal SHALL close
- **AND** no API call SHALL be made

### Requirement: Error notifications (added)

Vehicle store errors SHALL push toast notifications in addition to setting error state.

#### Scenario: API error shows toast
- **WHEN** a vehicle store action fails
- **THEN** a toast notification with `variant="error"` SHALL be pushed
- **AND** the error message from the API SHALL be displayed
