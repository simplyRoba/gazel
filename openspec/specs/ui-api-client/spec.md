## Purpose

Defines frontend behavior when communicating with the backend API, including successful responses, transport failures, and authentication-expiry navigation.

## Requirements

### Requirement: Frontend API response handling

The frontend SHALL handle backend responses according to their HTTP status and documented API contract without exposing response-processing failures to the user.

#### Scenario: Successful JSON response
- **WHEN** a frontend API operation receives a successful response containing JSON
- **THEN** the response data SHALL be made available to the requesting UI behavior

#### Scenario: No-content response
- **WHEN** a frontend API operation receives `204 No Content`
- **THEN** the operation SHALL complete successfully without attempting to read a response body

#### Scenario: Application error response
- **WHEN** a frontend API operation receives a non-success response containing the standard `code` and `message` fields
- **THEN** frontend error handling SHALL retain the HTTP status, error code, and fallback message for localization and presentation

#### Scenario: Non-JSON error response
- **WHEN** a frontend API operation receives a non-success response without a valid application error body
- **THEN** frontend error handling SHALL produce a generic unknown error using the HTTP status and status text
- **AND** failure to parse the response body SHALL NOT replace the original HTTP failure

#### Scenario: JSON request body
- **WHEN** the frontend sends a JSON request body to an API endpoint
- **THEN** the request SHALL use `Content-Type: application/json`
- **AND** the body SHALL conform to the endpoint's documented JSON contract

### Requirement: Authentication-required responses open the public login page

The frontend SHALL recognize exactly `401 Unauthorized` with error code `AUTHENTICATION_REQUIRED` and navigate the browser to `/login` instead of leaving an expired application view in an error-only state.

#### Scenario: API request requires authentication
- **WHEN** any frontend API request receives status `401` with code `AUTHENTICATION_REQUIRED`
- **THEN** the frontend SHALL navigate the top-level browser to `/login`
- **AND** include the current local UI path, query, and fragment as an encoded `return_to` value

#### Scenario: Export request requires authentication
- **WHEN** a frontend export request receives status `401` with code `AUTHENTICATION_REQUIRED`
- **THEN** it SHALL use the same `/login` navigation behavior as other API requests
- **AND** SHALL NOT attempt to parse or download the response as an export

#### Scenario: Concurrent expired API requests
- **WHEN** multiple requests receive `AUTHENTICATION_REQUIRED` before browser navigation completes
- **THEN** the frontend SHALL initiate login-page navigation only once

#### Scenario: Other API errors
- **WHEN** an API response has another status or error code, including a non-authentication `401`
- **THEN** the frontend SHALL NOT navigate to `/login`
- **AND** SHALL continue normal error handling

### Requirement: Frontend return target is encoded as a query value

The frontend SHALL derive `return_to` only from the current browser path, query, and fragment and SHALL encode the complete value as one login-page query parameter.

#### Scenario: Current settings route with fragment
- **WHEN** authentication expires at `/settings?tab=data#export`
- **THEN** navigation SHALL target `/login?return_to=%2Fsettings%3Ftab%3Ddata%23export`
- **AND** the fragment SHALL be carried inside the encoded query value rather than sent as the login page's own fragment
