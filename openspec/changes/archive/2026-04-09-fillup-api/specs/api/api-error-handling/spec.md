## MODIFIED Requirements

### Requirement: Vehicle error codes

The `default_message` function SHALL map vehicle-specific error codes to human-readable messages.

#### Scenario: Vehicle error code messages
- **WHEN** an error response uses a vehicle error code
- **THEN** the following mappings SHALL apply:
- **AND** `VEHICLE_NOT_FOUND` SHALL map to `"Vehicle not found."`
- **AND** `VEHICLE_NAME_REQUIRED` SHALL map to `"Vehicle name is required."`
- **AND** `VEHICLE_INVALID_FUEL_TYPE` SHALL map to `"Invalid fuel type."`
- **AND** `VEHICLE_INVALID_YEAR` SHALL map to `"Invalid year."`
- **AND** `VEHICLE_HAS_FILLUPS` SHALL map to `"Cannot delete vehicle with existing fill-ups."`

## ADDED Requirements

### Requirement: Fill-up error codes

The `default_message` function SHALL map fill-up-specific error codes to human-readable messages.

#### Scenario: Fill-up error code messages
- **WHEN** an error response uses a fill-up error code
- **THEN** the following mappings SHALL apply:
- **AND** `FILLUP_NOT_FOUND` SHALL map to `"Fill-up not found."`
- **AND** `FILLUP_DATE_REQUIRED` SHALL map to `"Fill-up date is required."`
- **AND** `FILLUP_FUEL_AMOUNT_REQUIRED` SHALL map to `"Fuel amount is required."`
- **AND** `FILLUP_INVALID_FUEL_AMOUNT` SHALL map to `"Fuel amount must be greater than zero."`
- **AND** `FILLUP_INVALID_ODOMETER` SHALL map to `"Odometer reading must not be less than the previous reading."`
- **AND** `FILLUP_INVALID_COST` SHALL map to `"Cost must not be negative."`
