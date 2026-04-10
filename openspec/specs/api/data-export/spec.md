## Purpose

Defines the data export API endpoints for exporting vehicles and fill-ups as portable JSON documents.

## Requirements

### Requirement: Export JSON schema

The export document SHALL be a JSON object with the following top-level fields:
- `version` (string): The application version (`CARGO_PKG_VERSION`) at build time.
- `exported_at` (string): ISO 8601 / RFC 3339 UTC timestamp of when the export was generated.
- `vehicles` (array): Array of vehicle objects, each embedding its fill-ups.

Each vehicle object SHALL contain all `Vehicle` API response fields except internal identifiers (`id`), plus a `fillups` array containing all `Fillup` API response fields except internal identifiers (`id`, `vehicle_id`) for that vehicle. Internal database IDs are not portable and are omitted from exports.

#### Scenario: Export document structure

- **WHEN** a client receives an export JSON document
- **THEN** the document SHALL contain `version`, `exported_at`, and `vehicles` keys
- **AND** `version` SHALL be a semver string matching the running server version
- **AND** `exported_at` SHALL be a valid RFC 3339 timestamp
- **AND** each entry in `vehicles` SHALL contain vehicle fields plus a `fillups` array

### Requirement: Full export endpoint

`GET /api/export` SHALL return all vehicles and their fill-ups as a JSON export document.

#### Scenario: Successful full export

- **WHEN** a `GET` request is made to `/api/export`
- **THEN** the response status SHALL be `200`
- **AND** the `Content-Type` header SHALL be `application/json`
- **AND** the response body SHALL be a valid export document containing all vehicles and all fill-ups

#### Scenario: Export with no data

- **WHEN** a `GET` request is made to `/api/export` and no vehicles exist
- **THEN** the response status SHALL be `200`
- **AND** the `vehicles` array SHALL be empty

#### Scenario: Fill-ups are ordered by date ascending within each vehicle

- **WHEN** a vehicle has multiple fill-ups
- **THEN** the fill-ups array in the export SHALL be sorted by `date` ascending (oldest first)

### Requirement: Single vehicle export endpoint

`GET /api/vehicles/:id/export` SHALL return a single vehicle and its fill-ups as a JSON export document.

#### Scenario: Successful single vehicle export

- **WHEN** a `GET` request is made to `/api/vehicles/:id/export` for an existing vehicle
- **THEN** the response status SHALL be `200`
- **AND** the response body SHALL be a valid export document
- **AND** the `vehicles` array SHALL contain exactly one vehicle matching the requested ID

#### Scenario: Vehicle not found

- **WHEN** a `GET` request is made to `/api/vehicles/:id/export` for a non-existent vehicle
- **THEN** the response status SHALL be `404`
- **AND** the response body SHALL follow the standard error shape with code `VEHICLE_NOT_FOUND`

### Requirement: Export content-disposition header

Export responses SHALL include a `Content-Disposition` header suggesting a filename.

#### Scenario: Full export filename

- **WHEN** a full export is requested via `GET /api/export`
- **THEN** the `Content-Disposition` header SHALL be `attachment; filename="gazel-export.json"`

#### Scenario: Single vehicle export filename

- **WHEN** a single vehicle export is requested
- **THEN** the `Content-Disposition` header SHALL be `attachment; filename="gazel-export-<vehicle-name>.json"` where `<vehicle-name>` is the vehicle's name in kebab-case
