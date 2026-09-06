## Purpose

Defines locale-aware display and input behavior for distances, volumes, efficiency, currency, decimal values, and dates.

## Requirements

### Requirement: Distance formatting

Displayed distances SHALL use locale-aware number formatting and the applicable unit suffix.

#### Scenario: Kilometers in English
- **WHEN** `1142.5` kilometers is displayed in English
- **THEN** it SHALL appear as `1,142.5 km`

#### Scenario: Kilometers in German
- **WHEN** `1142.5` kilometers is displayed in German
- **THEN** it SHALL appear as `1.142,5 km`

#### Scenario: Locale is unavailable
- **WHEN** `142.5` kilometers is displayed without an available locale preference
- **THEN** English formatting SHALL be used: `142.5 km`

### Requirement: Volume formatting

Displayed fuel volumes SHALL use locale-aware number formatting and the applicable unit suffix.

#### Scenario: Liters in German
- **WHEN** `45.2` liters is displayed in German
- **THEN** it SHALL appear as `45,2 L`

#### Scenario: Locale is unavailable
- **WHEN** `45.2` liters is displayed without an available locale preference
- **THEN** English formatting SHALL be used: `45.2 L`

### Requirement: Efficiency formatting

Displayed efficiency SHALL use locale-aware number formatting with the applicable distance-per-volume units.

#### Scenario: Metric efficiency in German
- **WHEN** `15.3` kilometers per liter is displayed in German
- **THEN** it SHALL appear as `15,3 km/L`

#### Scenario: Locale is unavailable
- **WHEN** `15.3` kilometers per liter is displayed without an available locale preference
- **THEN** English formatting SHALL be used: `15.3 km/L`

### Requirement: Currency formatting

Displayed monetary values SHALL use the applicable currency symbol and locale-aware number formatting.

#### Scenario: USD in English
- **WHEN** `1042.50` USD is displayed in English
- **THEN** it SHALL use English number conventions and the dollar sign, such as `$1,042.50`

#### Scenario: EUR in German
- **WHEN** `1042.50` EUR is displayed in German
- **THEN** it SHALL use German number conventions and the euro sign

#### Scenario: Locale is unavailable
- **WHEN** `42.50` USD is displayed without an available locale preference
- **THEN** English formatting SHALL be used: `$42.50`

### Requirement: Locale-aware decimal input

Numeric input SHALL accept dot or comma decimal separators, tolerate common grouping separators and stray symbols anywhere in the input, and interpret ambiguous separators using the active locale.

#### Scenario: Comma decimal
- **WHEN** the user enters `477,2`
- **THEN** it SHALL be interpreted as `477.2`

#### Scenario: Dot decimal
- **WHEN** the user enters `477.2`
- **THEN** it SHALL be interpreted as `477.2`

#### Scenario: Grouping and decimal separators
- **WHEN** the user enters `1.234,56` in German
- **THEN** it SHALL be interpreted as `1234.56`
- **WHEN** the user enters `1,234.56` in English
- **THEN** it SHALL be interpreted as `1234.56`

#### Scenario: Ambiguous single separator
- **WHEN** one separator is followed by exactly three digits, such as `234.567`
- **THEN** the locale SHALL determine whether it is decimal or grouping
- **AND** German SHALL interpret `234.567` as `234567`
- **AND** English SHALL interpret it as `234.567`

#### Scenario: Repeated separators
- **WHEN** one separator type occurs more than once, such as `1,234,567`
- **THEN** the separators SHALL be treated as grouping
- **AND** the value SHALL be interpreted as `1234567`

#### Scenario: Input has no numeric value
- **WHEN** input is empty, whitespace, null, undefined, or contains no digits
- **THEN** it SHALL be treated as invalid numeric input

### Requirement: Decimal input normalization

Valid numeric input SHALL be normalized to the active locale's display conventions when editing finishes.

#### Scenario: German normalization
- **WHEN** valid numeric input finishes editing in German
- **THEN** the displayed value SHALL use a comma decimal separator

#### Scenario: English normalization
- **WHEN** valid numeric input finishes editing in English
- **THEN** the displayed value SHALL use a dot decimal separator

### Requirement: Date formatting uses the active locale

Displayed dates SHALL use the current application locale.

#### Scenario: Dashboard date
- **WHEN** a fill-up date is displayed on the dashboard
- **THEN** it SHALL use the current application locale

#### Scenario: Chart-axis date
- **WHEN** a chart axis displays dates
- **THEN** they SHALL use the current application locale
