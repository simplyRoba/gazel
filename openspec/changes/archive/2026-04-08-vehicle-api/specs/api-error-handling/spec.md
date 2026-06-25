## ADDED Requirements

### Requirement: Nullable field deserialization helper

The error module SHALL provide a `deserialize_nullable` function for use with serde's `deserialize_with` attribute, enabling three-state PATCH semantics (`Option<Option<T>>`).

#### Scenario: Field absent from JSON
- **WHEN** a JSON request body omits a field annotated with `deserialize_nullable`
- **THEN** the field SHALL deserialize as `None` (outer Option)

#### Scenario: Field explicitly set to null
- **WHEN** a JSON request body sets a field to `null`
- **THEN** the field SHALL deserialize as `Some(None)` (inner Option is None)

#### Scenario: Field set to a value
- **WHEN** a JSON request body sets a field to a non-null value
- **THEN** the field SHALL deserialize as `Some(Some(value))`

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
