## Why

Gazel currently relies entirely on network placement or an authenticating reverse proxy, which makes safe direct self-hosting harder. Add a minimal, standards-compliant OIDC option that protects the embedded UI and API while preserving today’s zero-auth behavior by default.

## What Changes

- Add optional built-in OIDC authentication, disabled by default, using Authorization Code flow with discovery, state, nonce, PKCE, verified ID tokens, and issuer/audience/signature/expiry validation.
- Add secure backend-managed login transactions and authenticated sessions referenced by an authenticated-encrypted, HTTP-only cookie; OIDC tokens never enter browser storage and are discarded after verification.
- Protect the embedded UI and every `/api/*` route when authentication is enabled; keep `/health` and the login, callback, and logout endpoints public.
- Return a JSON `401 Unauthorized` for unauthenticated API requests while redirecting unauthenticated browser navigation to login.
- Add local logout that always destroys the Gazel session without depending on provider logout support.
- Add fail-closed startup validation and OIDC discovery for enabled configurations, using an explicit external URL to construct callback URLs rather than trusting proxy headers.
- Add local mock-provider coverage for protocol and session security behavior.
- Update deployment examples and user-facing security/configuration documentation.
- Do not add local users, registration, passwords, roles, permissions, groups, claim authorization, per-user data separation, account tables, Redis, or a frontend authentication library.

## Capabilities

### New Capabilities

- `core-authentication`: Optional OIDC login, callback validation, route protection, backend session lifecycle, and local logout.

### Modified Capabilities

- `core-configuration`: Add disabled-by-default auth settings and fail-closed validation for enabled authentication.
- `core-http-server`: Split public and protected routes and make embedded SPA fallback behavior authentication-aware.
- `api-health-check`: Require `/health` to remain public when authentication is enabled.
- `api-error-handling`: Add the stable JSON unauthorized response used by protected API routes.
- `ui-i18n`: Add localized entries for the new stable authentication-required error code.

## Impact

- **Rust**: new focused authentication module plus changes to configuration, startup, shared state, API errors, and router assembly.
- **HTTP surface**: new public `GET /auth/login`, `GET /auth/callback`, and `POST /auth/logout` endpoints; existing authenticated endpoint payloads remain unchanged.
- **Configuration**: `GAZEL_AUTH_ENABLED`, `GAZEL_AUTH_SECRET`, `GAZEL_EXTERNAL_URL`, `GAZEL_OIDC_ISSUER`, `GAZEL_OIDC_CLIENT_ID`, and `GAZEL_OIDC_CLIENT_SECRET`.
- **Dependencies**: maintained `openidconnect` for protocol validation and `tower-sessions` for encrypted-cookie-backed server-side sessions; no generalized authentication framework.
- **Tests/docs/deployment**: mock OIDC integration tests, configuration/session/router security tests, README updates, and Docker Compose examples.
- **Data model**: no users/accounts table and no per-user ownership changes; all successfully authenticated identities continue to share the same Gazel application data.
