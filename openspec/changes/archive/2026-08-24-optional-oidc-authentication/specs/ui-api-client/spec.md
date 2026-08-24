## ADDED Requirements

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
