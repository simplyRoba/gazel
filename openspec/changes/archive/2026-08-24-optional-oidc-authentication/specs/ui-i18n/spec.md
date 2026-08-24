## ADDED Requirements

### Requirement: Authentication translations
Every supported locale SHALL provide translations for the public login page, stable login states, authentication-required API error, and settings logout UI.

#### Scenario: English authentication keys
- **WHEN** `en.json` is loaded
- **THEN** it SHALL contain English values for `login.title`, `login.authenticationRequired`, `login.continueWith`, `login.error.authenticationFailed`, `login.error.providerUnavailable`, `login.error.configUnavailable`, and `login.loggedOut`
- **AND** SHALL contain `error.AUTHENTICATION_REQUIRED` with value `Authentication is required.`
- **AND** SHALL contain English values for `settings.authentication`, `settings.authentication.description`, and `settings.authentication.signOut`

#### Scenario: German authentication keys
- **WHEN** `de.json` is loaded
- **THEN** it SHALL contain equivalent German values for every new login and settings authentication key
- **AND** `login.continueWith` SHALL preserve its `{provider}` placeholder

#### Scenario: Translation parity remains complete
- **WHEN** the translation completeness test runs
- **THEN** every new authentication key SHALL exist in both locale files
