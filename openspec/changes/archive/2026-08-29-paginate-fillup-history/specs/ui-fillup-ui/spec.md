## MODIFIED Requirements

### Requirement: Fill-up API client types and functions

The API client SHALL export TypeScript types and functions for paginated fill-up retrieval and all fill-up CRUD operations.

#### Scenario: Fillup interface
- **WHEN** a `Fillup` object is used
- **THEN** it SHALL retain the existing fill-up fields and types

#### Scenario: FillupPage interface
- **WHEN** a fill-up list response is used
- **THEN** `FillupPage` SHALL contain `items: Fillup[]` and `next_cursor: string | null`

#### Scenario: CreateFillup and UpdateFillup interfaces
- **WHEN** create or update data is sent
- **THEN** it SHALL retain the existing required and optional fields
- **AND** it SHALL NOT include backend-populated `fuel_unit` or `currency`

#### Scenario: Fetch first page
- **WHEN** `fetchFillups(vehicleId)` is called without a cursor
- **THEN** it SHALL request the vehicle's first fill-up page and return `FillupPage`

#### Scenario: Fetch next page
- **WHEN** `fetchFillups(vehicleId, cursor)` is called with a server cursor
- **THEN** it SHALL pass the cursor unchanged and return the next `FillupPage`

#### Scenario: Fetch single fill-up
- **WHEN** `fetchFillup(vehicleId, fillupId)` is called
- **THEN** it SHALL return the requested `Fillup` through the existing endpoint

#### Scenario: Create, update, and delete fill-up
- **WHEN** an existing fill-up mutation client function is called
- **THEN** it SHALL retain its existing endpoint, payload, and return behavior

## ADDED Requirements

### Requirement: Incremental fill-up history state

The frontend SHALL maintain an independent paginated fill-up chain for each vehicle.

#### Scenario: Initial load
- **WHEN** a vehicle becomes active or is refreshed
- **THEN** its first page SHALL replace its loaded items and cursor
- **AND** a new request generation SHALL begin

#### Scenario: Continue loading
- **WHEN** a next cursor exists and continuation is requested
- **THEN** unseen items SHALL be appended in server order
- **AND** the returned cursor SHALL replace the previous cursor

#### Scenario: Guard continuation
- **WHEN** continuation is already active, paused after failure, or exhausted
- **THEN** another automatic continuation request SHALL NOT be sent

#### Scenario: Ignore stale response
- **WHEN** a response belongs to an earlier request generation
- **THEN** it SHALL NOT modify cached state

#### Scenario: Continuation failure and retry
- **WHEN** continuation fails
- **THEN** loaded items and the current cursor SHALL remain unchanged
- **AND** explicit retry SHALL request that cursor again

#### Scenario: Mutation refresh
- **WHEN** create, update, or delete succeeds
- **THEN** existing local cache behavior SHALL occur
- **AND** a fresh first-page generation SHALL replace stale pagination state

#### Scenario: Form helpers
- **WHEN** form assistance inspects fill-up history
- **THEN** it SHALL use currently loaded recent entries
- **AND** it SHALL NOT trigger continuation

### Requirement: Endless fill-up scrolling

The dashboard SHALL request older fill-up pages as the user approaches the end of loaded cards.

#### Scenario: Sentinel reached
- **WHEN** the sentinel approaches the active scroll viewport and a next cursor exists
- **THEN** continuation SHALL be requested once

#### Scenario: Loading more
- **WHEN** continuation is active
- **THEN** loaded cards SHALL remain visible
- **AND** a continuation loading indicator SHALL be shown

#### Scenario: Failed continuation
- **WHEN** continuation fails
- **THEN** automatic loading SHALL pause
- **AND** a retry action SHALL be shown

#### Scenario: Exhausted history
- **WHEN** no next cursor remains
- **THEN** the sentinel SHALL be removed
- **AND** scrolling SHALL trigger no further fill-up requests

#### Scenario: Observer lifecycle
- **WHEN** the active vehicle, scroll root, or component lifecycle changes
- **THEN** the old observer SHALL be disconnected before a new observer is created
