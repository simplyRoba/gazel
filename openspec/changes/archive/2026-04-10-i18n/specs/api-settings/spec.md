## MODIFIED Requirements

### Requirement: Valid field domains

The API SHALL accept only values from the following domains.

#### Scenario: Valid unit_system values

- **WHEN** `unit_system` is one of `metric`, `imperial`, `custom`
- **THEN** the value SHALL be accepted

#### Scenario: Valid distance_unit values

- **WHEN** `distance_unit` is one of `km`, `mi`
- **THEN** the value SHALL be accepted

#### Scenario: Valid volume_unit values

- **WHEN** `volume_unit` is one of `l`, `gal`
- **THEN** the value SHALL be accepted

#### Scenario: Valid color_mode values

- **WHEN** `color_mode` is one of `light`, `dark`, `system`
- **THEN** the value SHALL be accepted

#### Scenario: Valid currency values

- **WHEN** `currency` is one of `USD`, `EUR`
- **THEN** the value SHALL be accepted

#### Scenario: Valid locale values

- **WHEN** `locale` is one of `en`, `de`
- **THEN** the value SHALL be accepted
