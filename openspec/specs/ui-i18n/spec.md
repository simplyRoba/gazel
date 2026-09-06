## Purpose

Defines English and German localization behavior, including immediate locale changes, interpolation, fallback handling, localized errors, and translation completeness.

## Requirements

### Requirement: Supported locales

The application SHALL provide English (`en`) and German (`de`) translations for all user-visible linguistic text.

#### Scenario: English locale
- **WHEN** English is active
- **THEN** user-visible linguistic text SHALL be displayed in English

#### Scenario: German locale
- **WHEN** German is active
- **THEN** user-visible linguistic text SHALL be displayed in German

#### Scenario: Translation completeness
- **WHEN** the application is released
- **THEN** every user-visible linguistic message SHALL be available in both supported locales

### Requirement: Translation lookup behavior

Localized messages SHALL support parameter interpolation and deterministic fallback behavior.

#### Scenario: Parameterized message
- **WHEN** a localized message contains a named parameter such as `{count}`
- **THEN** the displayed message SHALL replace it with the supplied value

#### Scenario: German message is unavailable
- **WHEN** a message is unavailable in German
- **THEN** its English translation SHALL be displayed

#### Scenario: Message is unavailable in every locale
- **WHEN** a translation identifier has no message in any supported locale
- **THEN** the identifier itself SHALL be displayed as the final fallback

### Requirement: Locale changes apply immediately

The active locale SHALL follow the current application locale setting, and changing it SHALL update displayed translations without reloading the page.

#### Scenario: Locale changes
- **WHEN** the user changes the locale from English to German
- **THEN** displayed linguistic text SHALL update to German without a page reload

#### Scenario: Application starts
- **WHEN** the application initializes
- **THEN** displayed linguistic text SHALL use the current application locale setting

### Requirement: User-visible text is localized

All user-visible linguistic text SHALL use the active locale. Language-neutral metadata such as version numbers, repository URLs, license identifiers, years, owner names, and symbol-based legal notices SHALL NOT require translation.

#### Scenario: Navigation and forms
- **WHEN** navigation, labels, placeholders, or validation messages are displayed
- **THEN** their linguistic text SHALL use the active locale

#### Scenario: Empty states and notifications
- **WHEN** an empty state or notification is displayed
- **THEN** its linguistic text SHALL use the active locale

#### Scenario: Language-neutral metadata
- **WHEN** language-neutral metadata such as `© 2026 simplyRoba.` is displayed
- **THEN** the value MAY be rendered directly
- **AND** any accompanying descriptive label SHALL use the active locale

### Requirement: Error messages are localized

Known API error codes SHALL produce localized user-facing messages. Unknown codes SHALL use the safe fallback message returned by the API.

#### Scenario: Known error code
- **WHEN** an API operation fails with a code that has a message in the active locale
- **THEN** the localized message SHALL be displayed in error states and notifications

#### Scenario: Unknown error code
- **WHEN** an API operation fails with a code that has no localized message in any supported locale
- **THEN** the API response's fallback message SHALL be displayed

#### Scenario: API error translation completeness
- **WHEN** the application uses an API error code
- **THEN** that code SHALL have a localized message in every supported locale

### Requirement: Pull-to-refresh text is localized

Every supported locale SHALL provide messages for the pulling, release-to-refresh, and refreshing states.

#### Scenario: English pull-to-refresh states
- **WHEN** English is active
- **THEN** the states SHALL display `Pull to refresh`, `Release to refresh`, and `Refreshing...` respectively

#### Scenario: German pull-to-refresh states
- **WHEN** German is active
- **THEN** each pull-to-refresh state SHALL display its German translation

### Requirement: Authentication text is localized

Every supported locale SHALL provide messages for the public login page, stable login states, authentication-required errors, and settings logout controls.

#### Scenario: Public login and settings authentication text
- **WHEN** login or settings authentication controls are displayed
- **THEN** their titles, descriptions, actions, failure states, and signed-out state SHALL use the active locale
- **AND** the provider-specific login action SHALL interpolate the configured provider name into its localized text

#### Scenario: Authentication-required API error
- **WHEN** an authentication-required API error is shown in English
- **THEN** its message SHALL be `Authentication is required.`
- **AND** German SHALL provide the equivalent localized message
