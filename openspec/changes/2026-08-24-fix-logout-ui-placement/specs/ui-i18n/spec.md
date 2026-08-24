## MODIFIED Requirements

### Requirement: Authentication translations
Every supported locale SHALL provide translations for the public login page, stable login states, authentication-required API error, and persistent application chrome logout action.

#### Scenario: English authentication keys
- **WHEN** `en.json` is loaded
- **THEN** it SHALL contain English values for `login.title`, `login.authenticationRequired`, `login.continueWith`, `login.error.authenticationFailed`, `login.error.providerUnavailable`, `login.error.configUnavailable`, and `login.loggedOut`
- **AND** SHALL contain `error.AUTHENTICATION_REQUIRED` with value `Authentication is required.`
- **AND** SHALL retain `settings.authentication.signOut` with value `Sign out` for use by the application chrome

#### Scenario: German authentication keys
- **WHEN** `de.json` is loaded
- **THEN** it SHALL contain equivalent German values for every required login and application-chrome authentication key
- **AND** `login.continueWith` SHALL preserve its `{provider}` placeholder

#### Scenario: Translation parity remains complete
- **WHEN** the translation completeness test runs
- **THEN** every required authentication key SHALL exist in both locale files
