## MODIFIED Requirements

### Requirement: Authentication-required responses open the public login page

The frontend SHALL recognize exactly `401 Unauthorized` with error code `AUTHENTICATION_REQUIRED` and navigate the top-level browser to `/login` instead of leaving an expired application view in an error-only state, except that expiry handling SHALL NOT recursively navigate from `/login`, `/auth`, or any path under `/auth/`.

#### Scenario: API request requires authentication
- **WHEN** any frontend API request from a page other than `/login`, `/auth`, or a path under `/auth/` receives status `401` with code `AUTHENTICATION_REQUIRED`
- **THEN** the frontend SHALL navigate the top-level browser to `/login`
- **AND** include the validated current local UI path, query, and fragment as an encoded `return_to` value, using `/` if invalid

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

#### Scenario: Expiry handling does not recursively navigate from authentication pages
- **WHEN** a request receives exactly `401` with `AUTHENTICATION_REQUIRED` while the browser is on `/login`, `/auth`, or a path under `/auth/`
- **THEN** the frontend SHALL NOT initiate another login-page navigation
- **AND** SHALL retain safe error handling without automatically starting OIDC

### Requirement: Frontend return target is encoded as a query value

The frontend SHALL derive `return_to` only from the current browser path, query, and fragment and SHALL validate the serialized target using the same acceptance vectors as backend return navigation and the login page. It SHALL measure the 2,048 UTF-8 byte limit before validation-only percent decoding, use `/` for invalid targets, and encode the accepted original serialization as one login-page query parameter without normalization or recursive decoding.

#### Scenario: Current settings route with fragment
- **WHEN** authentication expires at `/settings?tab=data#export`
- **THEN** navigation SHALL target `/login?return_to=%2Fsettings%3Ftab%3Ddata%23export`
- **AND** the fragment SHALL be carried inside the encoded query value rather than sent as the login page's own fragment

#### Scenario: Ambiguous navigation vectors are rejected consistently
- **WHEN** the current serialized target has other than exactly one leading slash, malformed percent escapes or UTF-8, raw or decoded backslashes or Unicode controls U+0000–U+001F or U+007F–U+009F, dot path segments, encoded path separators `/`, `\`, `?`, or `#`, or residual percent or nested escapes in its path
- **THEN** expiry handling SHALL use `/login?return_to=%2F`, matching frontend login and backend validation

#### Scenario: Reserved prefix vectors match the application policy
- **WHEN** a target names `/api`, `/health`, `/auth`, or `/login`, or any descendant or encoded equivalent of those paths
- **THEN** return-target validation SHALL substitute `/`
- **AND** SHALL NOT reject an unrelated path such as `/apiary` or reject `/uploads` solely for that namespace in Gazel
- **AND** the separate recursive-navigation guard SHALL still suppress expiry navigation from `/login`, `/auth`, or paths under `/auth/`

#### Scenario: Serialized byte length governs acceptance
- **WHEN** a target exceeds 2,048 UTF-8 bytes in serialized form even though validation decoding would shorten it
- **THEN** expiry handling SHALL use `/login?return_to=%2F`
- **AND** subsequent query encoding size SHALL NOT itself disqualify a serialized target within the limit

#### Scenario: Escaped query and fragment retain serialization
- **WHEN** the current target is `/settings?label=100%25&next=a%2Fb#part%20one`
- **THEN** expiry handling SHALL encode that exact serialization as one `return_to` query value
- **AND** the login page and backend SHALL recover that same serialized target with one form-decoding of the query value
- **AND** validation SHALL NOT reject a valid query escape merely because it decodes to a literal percent
