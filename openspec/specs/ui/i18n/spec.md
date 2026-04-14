## Purpose

Defines the internationalization (i18n) system for the UI, including translation file structure, lookup functions, reactive locale binding, and error message resolution.

## Requirements

### Requirement: Translation file structure

The app SHALL store translations as static JSON files in `ui/src/lib/i18n/`, one file per supported locale (`en.json`, `de.json`). Each file SHALL contain a flat object mapping dot-separated keys to translated strings.

#### Scenario: English translation file exists

- **WHEN** the app is built
- **THEN** `ui/src/lib/i18n/en.json` SHALL exist with all translation keys

#### Scenario: German translation file exists

- **WHEN** the app is built
- **THEN** `ui/src/lib/i18n/de.json` SHALL exist with all translation keys matching those in `en.json`

#### Scenario: Translation key format

- **WHEN** a translation key is defined
- **THEN** it SHALL use dot-separated segments describing the component and purpose (e.g., `nav.dashboard`, `fillup.form.date`, `error.VEHICLE_NOT_FOUND`)

### Requirement: Translation lookup function

The app SHALL provide a `t(key, params?)` function that looks up a translation key for the active locale and returns the translated string.

#### Scenario: Simple key lookup

- **WHEN** `t('nav.dashboard')` is called with locale `en`
- **THEN** the result SHALL be `"Dashboard"`

#### Scenario: Key lookup in German

- **WHEN** `t('nav.dashboard')` is called with locale `de`
- **THEN** the result SHALL be `"Dashboard"` (or the German translation)

#### Scenario: Parameterized string

- **WHEN** `t('import.summary.vehicles', { count: 3 })` is called
- **THEN** the result SHALL replace `{count}` with `3` in the translated string

#### Scenario: Missing key falls back to English

- **WHEN** `t('some.key')` is called with locale `de` and the key is missing from `de.json`
- **THEN** the result SHALL return the value from `en.json`

#### Scenario: Missing key in all locales

- **WHEN** `t('nonexistent.key')` is called and the key does not exist in any locale
- **THEN** the result SHALL return the key itself (e.g., `"nonexistent.key"`)

### Requirement: Reactive locale binding

The translation system SHALL reactively update all translated strings when the active locale changes.

#### Scenario: Locale change triggers re-render

- **WHEN** the user switches from `en` to `de` in settings
- **THEN** all components using `t()` SHALL re-render with German strings without a page reload

#### Scenario: Locale is driven by settings store

- **WHEN** the app initializes
- **THEN** the active locale SHALL be set from `settings.locale`
- **AND** the `t()` function SHALL use this locale for all lookups

### Requirement: All UI strings use translation keys

All user-visible strings in Svelte components, pages, and stores SHALL use the `t()` function instead of hardcoded English text.

#### Scenario: Navigation labels

- **WHEN** the app shell renders navigation items
- **THEN** the labels SHALL be rendered via `t('nav.dashboard')`, `t('nav.settings')`, etc.

#### Scenario: Form labels and validation messages

- **WHEN** a form renders labels, placeholders, and validation errors
- **THEN** all text SHALL be rendered via `t()` calls with appropriate keys

#### Scenario: Empty states

- **WHEN** an empty state component renders
- **THEN** the title and description SHALL be rendered via `t()` calls

#### Scenario: Toast notifications

- **WHEN** a toast notification is shown
- **THEN** the message SHALL be rendered via `t()` calls or `resolveError()`

### Requirement: Error message resolution

The app SHALL provide a `resolveError(error, t)` function in `ui/src/lib/i18n/errors.ts` that maps `ApiError.code` values to localized messages via translation keys.

#### Scenario: Known error code

- **WHEN** `resolveError(error, t)` is called with `error.code === 'VEHICLE_NOT_FOUND'`
- **THEN** the result SHALL be `t('error.VEHICLE_NOT_FOUND')`

#### Scenario: Unknown error code with message

- **WHEN** `resolveError(error, t)` is called with an error code that has no translation key
- **THEN** the result SHALL fall back to `error.message`

#### Scenario: Store error handling uses resolveError

- **WHEN** a store catches an `ApiError`
- **THEN** it SHALL pass the error through `resolveError()` to get the user-facing message
- **AND** the localized message SHALL be used for toast notifications and error state

### Requirement: Translation completeness test

A vitest test SHALL verify that all translation files contain the same set of keys.

#### Scenario: All keys present in German

- **WHEN** the translation completeness test runs
- **THEN** every key in `en.json` SHALL also exist in `de.json`

#### Scenario: No extra keys in non-primary locales

- **WHEN** the translation completeness test runs
- **THEN** `de.json` SHALL NOT contain keys that are absent from `en.json`

#### Scenario: Test fails on missing key

- **WHEN** a developer adds a new key to `en.json` without adding it to `de.json`
- **THEN** the completeness test SHALL fail with a message indicating the missing key

### Requirement: Pull-to-refresh translation keys

The translation files SHALL include keys for pull-to-refresh indicator labels.

#### Scenario: English pull-to-refresh keys

- **WHEN** `en.json` is loaded
- **THEN** it SHALL contain `"pullToRefresh.pulling"` with value `"Pull to refresh"`
- **AND** it SHALL contain `"pullToRefresh.release"` with value `"Release to refresh"`
- **AND** it SHALL contain `"pullToRefresh.refreshing"` with value `"Refreshing..."`

#### Scenario: German pull-to-refresh keys

- **WHEN** `de.json` is loaded
- **THEN** it SHALL contain `"pullToRefresh.pulling"` with the German translation
- **AND** it SHALL contain `"pullToRefresh.release"` with the German translation
- **AND** it SHALL contain `"pullToRefresh.refreshing"` with the German translation
