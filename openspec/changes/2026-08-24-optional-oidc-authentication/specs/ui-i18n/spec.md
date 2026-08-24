## ADDED Requirements

### Requirement: Authentication error translation
Every supported locale SHALL provide a translation for the stable `AUTHENTICATION_REQUIRED` API error code.

#### Scenario: English authentication error
- **WHEN** `en.json` is loaded
- **THEN** it SHALL contain `error.AUTHENTICATION_REQUIRED` with value `Authentication is required.`

#### Scenario: German authentication error
- **WHEN** `de.json` is loaded
- **THEN** it SHALL contain `error.AUTHENTICATION_REQUIRED` with an equivalent German translation

#### Scenario: Translation parity remains complete
- **WHEN** the translation completeness test runs
- **THEN** `error.AUTHENTICATION_REQUIRED` SHALL exist in both locale files
