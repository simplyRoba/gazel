## Purpose

Frontend API client: a centralized HTTP request helper, an ApiError class, typed vehicle API functions, and vehicle type definitions.

## Requirements

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

### Requirement: Authentication-required responses open the public login page
The frontend API client SHALL recognize exactly `401 Unauthorized` with error code `AUTHENTICATION_REQUIRED` and navigate the browser to `/login` instead of leaving an expired SPA in an error-only state.

#### Scenario: Central request receives authentication-required response
- **WHEN** the centralized `request()` helper receives status `401` with code `AUTHENTICATION_REQUIRED`
- **THEN** it SHALL navigate the top-level browser to `/login`
- **AND** include the current local UI path, query, and fragment as an encoded `return_to` value

#### Scenario: Export receives authentication-required response
- **WHEN** `exportAll()` or `exportVehicle()` receives status `401` with code `AUTHENTICATION_REQUIRED`
- **THEN** it SHALL use the same `/login` navigation behavior as the centralized request helper
- **AND** SHALL NOT attempt to parse or download the response as an export

#### Scenario: Concurrent expired API requests
- **WHEN** multiple requests receive `AUTHENTICATION_REQUIRED` before browser navigation completes
- **THEN** the API client SHALL initiate login-page navigation only once

#### Scenario: Other API errors
- **WHEN** an API response has another status or error code, including a non-authentication 401
- **THEN** the API client SHALL NOT navigate to `/login`
- **AND** SHALL continue to throw the normal typed `ApiError`

### Requirement: Frontend return target is encoded as a query value
The frontend SHALL derive `return_to` only from the current browser path/query/fragment and SHALL encode the complete value as one login-page query parameter.

#### Scenario: Current settings route with fragment
- **WHEN** authentication expires at `/settings?tab=data#export`
- **THEN** navigation SHALL target `/login?return_to=%2Fsettings%3Ftab%3Ddata%23export`
- **AND** the fragment SHALL be carried inside the encoded query value rather than sent as the login page's own fragment
