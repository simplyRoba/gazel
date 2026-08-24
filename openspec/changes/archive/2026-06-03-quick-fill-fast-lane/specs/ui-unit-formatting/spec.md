## ADDED Requirements

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
