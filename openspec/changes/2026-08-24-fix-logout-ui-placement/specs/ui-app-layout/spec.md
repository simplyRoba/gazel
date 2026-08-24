## ADDED Requirements

### Requirement: Persistent navigation chrome exposes local logout when authentication is enabled
The protected application shell SHALL provide the existing translated Sign out action in its persistent navigation chrome only when the authenticated app-info response reports that built-in authentication is enabled. The action SHALL remain separate from route navigation, SHALL use neutral secondary styling rather than destructive styling, and SHALL NOT be wrapped in an account, profile, identity, or user menu.

#### Scenario: Authentication-enabled application chrome
- **WHEN** `/api/info` includes `auth_enabled: true`
- **THEN** the persistent navigation chrome SHALL display the translated Sign out action
- **AND** sidebar layouts SHALL position it at the bottom below and visually separated from the normal navigation items
- **AND** responsive navigation variants SHALL keep the action in persistent application chrome
- **AND** the action SHALL be visually distinct from route navigation without appearing destructive

#### Scenario: Authentication-disabled application chrome
- **WHEN** `/api/info` omits `auth_enabled`
- **THEN** the application chrome SHALL NOT display the Sign out action
- **AND** the existing disabled-mode navigation SHALL remain unchanged

#### Scenario: Local logout from application chrome
- **WHEN** the user activates Sign out in the application chrome
- **THEN** the browser SHALL submit `POST /auth/logout` as a top-level form navigation
- **AND** SHALL follow the existing response redirect to `/login?logged_out=1`
- **AND** the public login page SHALL not automatically initiate provider login

#### Scenario: Logout remains identity-neutral
- **WHEN** the Sign out action is displayed
- **THEN** the application chrome SHALL NOT display identity, profile, local-user, or account concepts
- **AND** SHALL NOT add an account menu solely to contain the action
