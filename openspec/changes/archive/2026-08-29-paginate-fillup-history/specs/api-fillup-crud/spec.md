## MODIFIED Requirements

### Requirement: List fill-ups for a vehicle

The API SHALL return a cursor-paginated page of fill-ups for a vehicle, ordered by date descending and ID descending.

#### Scenario: First page
- **WHEN** `GET /api/vehicles/{vehicle_id}/fillups` is requested without a cursor for an existing vehicle
- **THEN** the response SHALL be `200 OK`
- **AND** `items` SHALL contain at most 25 fill-ups by default
- **AND** `next_cursor` SHALL be an opaque string when older entries exist or `null` otherwise

#### Scenario: Requested page size
- **WHEN** a valid integer `limit` from 1 through 100 is supplied
- **THEN** `items` SHALL contain at most that many fill-ups

#### Scenario: Continue listing
- **WHEN** a returned cursor is supplied to the next request
- **THEN** the response SHALL contain the next older items in the same order
- **AND** no item from the preceding page SHALL be repeated

#### Scenario: Empty or terminal page
- **WHEN** no additional fill-ups exist
- **THEN** `items` SHALL be empty or contain the remaining items
- **AND** `next_cursor` SHALL be `null`

#### Scenario: Invalid pagination input
- **WHEN** `limit` is invalid
- **THEN** the API SHALL return `400 Bad Request` with code `FILLUP_INVALID_PAGE_LIMIT`
- **WHEN** `cursor` is malformed
- **THEN** the API SHALL return `400 Bad Request` with code `FILLUP_INVALID_CURSOR`

#### Scenario: List for non-existent vehicle
- **WHEN** the list is requested for a vehicle that does not exist
- **THEN** the response SHALL be `404 Not Found`
- **AND** the body SHALL contain `"code": "VEHICLE_NOT_FOUND"`
