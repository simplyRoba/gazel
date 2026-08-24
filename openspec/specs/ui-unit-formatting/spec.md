## Purpose

Defines pure formatting functions for distances, volumes, fuel efficiency, and currency values used throughout the UI.

## Requirements

### Requirement: Distance formatting

The app SHALL provide a `formatDistance(value, unit, locale?)` function that formats a numeric distance value with the appropriate unit suffix, using locale-aware number formatting.

#### Scenario: Format kilometers with English locale

- **WHEN** `formatDistance(1142.5, 'km', 'en')` is called
- **THEN** the result SHALL be `"1,142.5 km"` (dot decimal, comma grouping)

#### Scenario: Format kilometers with German locale

- **WHEN** `formatDistance(1142.5, 'km', 'de')` is called
- **THEN** the result SHALL be `"1.142,5 km"` (comma decimal, dot grouping)

#### Scenario: Default locale is English

- **WHEN** `formatDistance(142.5, 'km')` is called without a locale
- **THEN** the result SHALL use English formatting: `"142.5 km"`

### Requirement: Volume formatting

The app SHALL provide a `formatVolume(value, unit, locale?)` function that formats a numeric volume value with locale-aware number formatting.

#### Scenario: Format liters with German locale

- **WHEN** `formatVolume(45.2, 'l', 'de')` is called
- **THEN** the result SHALL be `"45,2 L"` (comma decimal separator)

#### Scenario: Default locale is English

- **WHEN** `formatVolume(45.2, 'l')` is called without a locale
- **THEN** the result SHALL use English formatting: `"45.2 L"`

### Requirement: Efficiency formatting

The app SHALL provide a `formatEfficiency(value, distanceUnit, volumeUnit, locale?)` function with locale-aware number formatting.

#### Scenario: Metric efficiency with German locale

- **WHEN** `formatEfficiency(15.3, 'km', 'l', 'de')` is called
- **THEN** the result SHALL be `"15,3 km/L"` (comma decimal)

#### Scenario: Default locale is English

- **WHEN** `formatEfficiency(15.3, 'km', 'l')` is called without a locale
- **THEN** the result SHALL use English formatting: `"15.3 km/L"`

### Requirement: Currency formatting

The app SHALL provide a `formatCurrency(value, currency, locale?)` function that formats a monetary value with locale-aware number formatting.

#### Scenario: Format USD with English locale

- **WHEN** `formatCurrency(1042.50, 'USD', 'en')` is called
- **THEN** the result SHALL use locale-aware formatting with dollar sign (e.g., `"$1,042.50"`)

#### Scenario: Format EUR with German locale

- **WHEN** `formatCurrency(1042.50, 'EUR', 'de')` is called
- **THEN** the result SHALL use locale-aware formatting with euro sign and German number conventions

#### Scenario: Default locale is English

- **WHEN** `formatCurrency(42.50, 'USD')` is called without a locale
- **THEN** the result SHALL use English formatting: `"$42.50"`

### Requirement: Locale-aware decimal parsing

The app SHALL provide a `parseDecimal(value, locale?)` function that parses a user-entered numeric string into a number, accepting both dot (`.`) and comma (`,`) as the decimal separator and tolerating grouping separators and stray symbols.

#### Scenario: Parse comma decimal

- **WHEN** `parseDecimal('477,2')` is called
- **THEN** the result SHALL be `477.2`

#### Scenario: Parse dot decimal

- **WHEN** `parseDecimal('477.2')` is called
- **THEN** the result SHALL be `477.2`

#### Scenario: Both separators present (grouping + decimal)

- **WHEN** `parseDecimal('1.234,56', 'de')` is called
- **THEN** the result SHALL be `1234.56`
- **WHEN** `parseDecimal('1,234.56', 'en')` is called
- **THEN** the result SHALL be `1234.56`

#### Scenario: Ambiguous single separator resolved by locale

- **WHEN** a single separator is followed by exactly three digits (e.g. `234.567`)
- **THEN** the active locale's decimal separator SHALL decide whether it is a decimal point or a grouping separator (`234.567` in `de` SHALL be `234567`; in `en` SHALL be `234.567`)

#### Scenario: Repeated separators are grouping

- **WHEN** a single separator type occurs more than once (e.g. `1,234,567`)
- **THEN** it SHALL be treated as grouping and removed (`1234567`)

#### Scenario: Numbers and junk

- **WHEN** a number is passed
- **THEN** it SHALL be returned unchanged
- **WHEN** the input is empty, whitespace, null, undefined, or contains no digits
- **THEN** the result SHALL be `NaN`

### Requirement: Decimal input normalization

The app SHALL provide a helper to normalize a parsed numeric value back into the user's locale display string for use on input blur.

#### Scenario: Normalize for German locale

- **WHEN** a valid numeric value is normalized for the `de` locale
- **THEN** the result SHALL use comma as the decimal separator

#### Scenario: Normalize for English locale

- **WHEN** a valid numeric value is normalized for the `en` locale
- **THEN** the result SHALL use dot as the decimal separator

### Requirement: Pure function design

All formatting functions SHALL be pure functions that take explicit parameters and do not read from any store or global state.

#### Scenario: No implicit dependencies

- **WHEN** any formatting function is called with explicit parameters (including locale)
- **THEN** it SHALL return a deterministic result based solely on its arguments
- **AND** it SHALL NOT import or reference any Svelte store

### Requirement: Date formatting uses locale

All date formatting in the app SHALL use the active locale from settings.

#### Scenario: Dashboard date formatting

- **WHEN** a fill-up date is displayed on the dashboard
- **THEN** it SHALL be formatted using `toLocaleDateString()` with the settings locale

#### Scenario: Chart axis date formatting

- **WHEN** chart axes display dates
- **THEN** they SHALL use the settings locale instead of hardcoded `"en"`
