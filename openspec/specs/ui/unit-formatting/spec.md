## Purpose

Defines pure formatting functions for distances, volumes, fuel efficiency, and currency values used throughout the UI.

## Requirements

### Requirement: Distance formatting

The app SHALL provide a `formatDistance(value, unit)` function that formats a numeric distance value with the appropriate unit suffix.

#### Scenario: Format kilometers

- **WHEN** `formatDistance(142.5, 'km')` is called
- **THEN** the result SHALL be `"142.5 km"`

#### Scenario: Format miles

- **WHEN** `formatDistance(88.6, 'mi')` is called
- **THEN** the result SHALL be `"88.6 mi"`

#### Scenario: Rounds to one decimal place

- **WHEN** `formatDistance(100.456, 'km')` is called
- **THEN** the result SHALL be `"100.5 km"`

### Requirement: Volume formatting

The app SHALL provide a `formatVolume(value, unit)` function that formats a numeric volume value with the appropriate unit suffix.

#### Scenario: Format liters

- **WHEN** `formatVolume(45.2, 'l')` is called
- **THEN** the result SHALL be `"45.2 L"`

#### Scenario: Format gallons

- **WHEN** `formatVolume(12.0, 'gal')` is called
- **THEN** the result SHALL be `"12.0 gal"`

#### Scenario: Rounds to one decimal place

- **WHEN** `formatVolume(33.789, 'l')` is called
- **THEN** the result SHALL be `"33.8 L"`

### Requirement: Efficiency formatting

The app SHALL provide a `formatEfficiency(value, distanceUnit, volumeUnit)` function that formats fuel efficiency with the correct composite unit.

#### Scenario: Metric efficiency (km/L)

- **WHEN** `formatEfficiency(15.3, 'km', 'l')` is called
- **THEN** the result SHALL be `"15.3 km/L"`

#### Scenario: Imperial efficiency (mi/gal = MPG)

- **WHEN** `formatEfficiency(32.1, 'mi', 'gal')` is called
- **THEN** the result SHALL be `"32.1 mpg"`

#### Scenario: Mixed units

- **WHEN** `formatEfficiency(20.0, 'km', 'gal')` is called
- **THEN** the result SHALL be `"20.0 km/gal"`

#### Scenario: Rounds to one decimal place

- **WHEN** `formatEfficiency(28.456, 'mi', 'gal')` is called
- **THEN** the result SHALL be `"28.5 mpg"`

### Requirement: Currency formatting

The app SHALL provide a `formatCurrency(value, currency)` function that formats a monetary value with the appropriate currency symbol or code.

#### Scenario: Format USD

- **WHEN** `formatCurrency(42.50, 'USD')` is called
- **THEN** the result SHALL be `"$42.50"`

#### Scenario: Format EUR

- **WHEN** `formatCurrency(42.50, 'EUR')` is called
- **THEN** the result SHALL be `"\u20AC42.50"`

#### Scenario: Format GBP

- **WHEN** `formatCurrency(42.50, 'GBP')` is called
- **THEN** the result SHALL be `"\u00A342.50"`

#### Scenario: Rounds to two decimal places

- **WHEN** `formatCurrency(10.999, 'USD')` is called
- **THEN** the result SHALL be `"$11.00"`

#### Scenario: Unknown currency falls back to code prefix

- **WHEN** `formatCurrency(42.50, 'XYZ')` is called
- **THEN** the result SHALL be `"XYZ 42.50"`

### Requirement: Pure function design

All formatting functions SHALL be pure functions that take explicit parameters and do not read from any store or global state.

#### Scenario: No implicit dependencies

- **WHEN** any formatting function is called with explicit parameters
- **THEN** it SHALL return a deterministic result based solely on its arguments
- **AND** it SHALL NOT import or reference any Svelte store
