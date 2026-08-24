## Why

Gazel currently relies entirely on network placement or an authenticating reverse proxy, which makes safe direct self-hosting harder. Add a minimal, standards-compliant OIDC option with a branded public login experience while preserving today’s unauthenticated application/API behavior by default, apart from one inert public auth-config endpoint.

## What Changes

- Add optional built-in OIDC authentication, disabled by default, using Authorization Code flow with discovery, state, nonce, PKCE, verified ID tokens, and issuer/audience/signature/expiry validation.
- Add a dedicated public `/login` Svelte page with Gazel branding, concise authentication-required/error/signed-out states, and exactly one OIDC button.
- Add optional `GAZEL_OIDC_PROVIDER_NAME`, defaulting to `OpenID Connect`, for the login button label; expose only that display value through a public auth-config endpoint.
- Negotiate standards-defined `client_secret_basic` and `client_secret_post` token-endpoint authentication from discovery metadata, failing startup when neither is usable.
- Add secure backend-managed login transactions and authenticated sessions referenced by an authenticated-encrypted, HTTP-only cookie; generate the cookie key per process and keep OIDC tokens entirely backend-side.
- Protect every application UI route plus `/api` and `/api/*` when authentication is enabled; keep `/login`, the static assets required to render it, `/health`, and login/callback/logout/config endpoints public.
- Return JSON `401 Unauthorized` to unauthenticated API clients while redirecting browser navigation and an already-open SPA to `/login`.
- Preserve a validated local UI `return_to` through the login page and OIDC flow: backend navigation captures only request path/query, while SPA expiry recovery may additionally encode `location.hash` as query-parameter data; default targets over 2,048 decoded UTF-8 bytes to `/`.
- Return every failed callback to `/login` with a stable error state and an always-present safe `return_to`, defaulting to encoded `/`.
- Redirect an already-authenticated `GET /login` to `/` instead of rendering an authentication-required page.
- Add local logout that always destroys the Gazel session without depending on provider logout support, plus a settings-page action returning to the public login page’s signed-out state.
- Add fail-closed startup validation and OIDC discovery using an explicit external URL rather than trusting proxy headers.
- Cache startup discovery metadata and keys, refreshing only one JWKS generation when ID-token signature/key verification requires it.
- Add local mock-provider and frontend coverage for protocol, session, expiration, login-page, and reauthentication behavior.
- Update deployment examples and user-facing security/configuration documentation.
- Do not add local users, registration, usernames, passwords, roles, permissions, groups, claim authorization, per-user data separation, account tables, Redis, or a frontend authentication framework.

## Capabilities

### New Capabilities

- `core-authentication`: Optional OIDC initiation/callback validation, route protection, backend sessions, safe return navigation, public auth metadata, and local logout.
- `ui-login`: Public branded login page, provider-labelled OIDC action, safe return propagation, and authentication failure/signed-out states.

### Modified Capabilities

- `core-configuration`: Add disabled-by-default auth settings, provider display name, and fail-closed enabled validation.
- `core-http-server`: Split public/protected routes and make SPA/static serving authentication-aware.
- `api-health-check`: Require `/health` to remain public when authentication is enabled.
- `api-error-handling`: Add the stable JSON unauthorized response used by protected API routes.
- `ui-api-client`: Send the open SPA to `/login` on the exact authentication-required API response, including export requests.
- `ui-app-layout`: Render `/login` outside the protected application shell and skip protected hydration there.
- `ui-settings`: Show a logout action only when built-in authentication is enabled.
- `ui-i18n`: Add localized login, authentication-required, error, signed-out, and logout strings.

## Impact

- **Rust**: focused authentication module plus changes to configuration, startup, shared state, API errors, app-info output, public auth config, static routing, and router assembly.
- **UI**: new `ui/src/routes/login/+page.svelte`, login-aware root layout, centralized session-expiry handling, and conditional settings logout; no frontend auth library or browser token storage.
- **HTTP surface**: public `GET /login`, `GET /auth/config`, `GET /auth/login`, `GET /auth/callback`, `POST /auth/logout`, and exact login-page assets when enabled; inert `GET /auth/config` is also public when disabled; existing domain endpoint payloads remain unchanged.
- **Configuration**: `GAZEL_AUTH_ENABLED`, `GAZEL_EXTERNAL_URL`, `GAZEL_OIDC_ISSUER`, `GAZEL_OIDC_CLIENT_ID`, `GAZEL_OIDC_CLIENT_SECRET`, and optional `GAZEL_OIDC_PROVIDER_NAME`.
- **Dependencies**: maintained `openidconnect` for protocol validation and `tower-sessions` for private-cookie-referenced server-side sessions; no generalized authentication framework.
- **Tests/docs/deployment**: mock OIDC integration tests, configuration/session/router tests, login/API-client/settings component tests, README updates, and Docker Compose examples.
- **Data model**: no users/accounts/session table and no per-user ownership changes; all successfully authenticated identities share the same Gazel application data.
