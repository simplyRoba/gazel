## ADDED Requirements

### Requirement: Centralized HTTP request helper

The API client SHALL provide a generic `request<T>()` function that handles JSON serialization, response parsing, and error mapping for all backend API calls.

#### Scenario: Successful JSON response
- **WHEN** `request()` receives a response with status 200-299
- **THEN** the response body SHALL be parsed as JSON and returned as type `T`

#### Scenario: No-content response
- **WHEN** `request()` receives a `204 No Content` response
- **THEN** it SHALL return `undefined`

#### Scenario: Error response with code
- **WHEN** `request()` receives a non-OK response with a JSON body containing `code` and `message`
- **THEN** it SHALL throw an `ApiError` with the `status`, `code`, and `message` from the response

#### Scenario: Error response without JSON body
- **WHEN** `request()` receives a non-OK response without a valid JSON body
- **THEN** it SHALL throw an `ApiError` with the HTTP status, code `"UNKNOWN_ERROR"`, and the status text as message

#### Scenario: Request with JSON body
- **WHEN** `request()` is called with a body object
- **THEN** it SHALL set `Content-Type: application/json` and serialize the body with `JSON.stringify()`

### Requirement: ApiError class

The API client SHALL export an `ApiError` class extending `Error` with `status` (number) and `code` (string) properties.

#### Scenario: ApiError construction
- **WHEN** an `ApiError` is constructed with status, code, and message
- **THEN** `error.status` SHALL be the HTTP status code
- **AND** `error.code` SHALL be the machine-readable error code
- **AND** `error.message` SHALL be the human-readable message

### Requirement: Typed vehicle API functions

The API client SHALL export typed functions for all vehicle CRUD operations.

#### Scenario: Fetch all vehicles
- **WHEN** `fetchVehicles()` is called
- **THEN** it SHALL send `GET /api/vehicles` and return `Vehicle[]`

#### Scenario: Fetch single vehicle
- **WHEN** `fetchVehicle(id)` is called
- **THEN** it SHALL send `GET /api/vehicles/{id}` and return `Vehicle`

#### Scenario: Create vehicle
- **WHEN** `createVehicle(data)` is called
- **THEN** it SHALL send `POST /api/vehicles` with the data and return the created `Vehicle`

#### Scenario: Update vehicle
- **WHEN** `updateVehicle(id, data)` is called
- **THEN** it SHALL send `PUT /api/vehicles/{id}` with the data and return the updated `Vehicle`

#### Scenario: Delete vehicle
- **WHEN** `deleteVehicle(id)` is called
- **THEN** it SHALL send `DELETE /api/vehicles/{id}` and return `void`

### Requirement: Vehicle type definitions

The API client SHALL export TypeScript interfaces matching the backend API contract.

#### Scenario: Vehicle interface
- **WHEN** a `Vehicle` object is used in the frontend
- **THEN** it SHALL have fields: `id` (number), `name` (string), `make` (string | null), `model` (string | null), `year` (number | null), `fuel_type` (string), `notes` (string | null), `created_at` (string), `updated_at` (string)

#### Scenario: CreateVehicle interface
- **WHEN** a `CreateVehicle` object is sent to the API
- **THEN** it SHALL have `name` (string) as required, and `make`, `model`, `year`, `fuel_type`, `notes` as optional
