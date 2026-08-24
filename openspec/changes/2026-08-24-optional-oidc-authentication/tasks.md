## 1. Dependencies and configuration

- [ ] 1.1 Add maintained `openidconnect` and `tower-sessions` private-cookie support plus only directly used URL/time utilities; verify dependency resolution and backend-only `cargo check` succeed without an auth framework or Base64 secret dependency
- [ ] 1.2 Add configuration tests first for disabled defaults, explicit false, malformed enable flags, every missing/empty enabled value, HTTPS/loopback URL policy, forbidden URL components/paths, exact callback construction, default/custom provider name, and empty/control/overlength provider names
- [ ] 1.3 Implement fallible typed auth configuration with optional `GAZEL_OIDC_PROVIDER_NAME=OpenID Connect` default while preserving legacy port/database/log fallbacks and disabled application/API behavior; verify all config tests pass
- [ ] 1.4 Add tests for secure per-process private-cookie key generation and failure, then implement enabled-startup generation without an operator-managed cookie secret setting; verify key/store replacement invalidates prior cookies

## 2. OIDC runtime and protocol validation

- [ ] 2.1 Add a local test OIDC provider with discovery, authorization, token, and JWKS endpoints plus crate-signed ID-token fixtures and controllable malformed/invalid responses; verify it runs entirely on loopback without a real IdP
- [ ] 2.2 Add discovery/client tests first for `client_secret_basic`-only, `client_secret_post`-only, omitted, both, and unsupported-only token-auth metadata, plus redirects, insecure endpoints, wrong issuer, missing code flow/endpoints, and empty/unusable JWKS
- [ ] 2.3 Implement the guarded no-redirect Rustls client, startup discovery/JWKS validation, cached runtime, and explicit Basic/Post selection; verify omitted metadata defaults to Basic, both-supported metadata prefers Basic, and Post-only metadata selects request-body authentication
- [ ] 2.4 Add login tests first for state, nonce, S256 PKCE, external callback despite forwarded headers, fresh/replaced transactions, already-authenticated behavior, safe path/query `return_to`, and a SPA hash supplied only as `%23` query-parameter data; never model a browser fragment as part of the backend request target
- [ ] 2.5 Add validator tests for absolute/protocol-relative/reserved targets and percent-decoded backslash/control characters before redirect serialization; implement login initiation and backend-only return-target storage
- [ ] 2.6 Add callback tests first for valid `client_secret_basic` and `client_secret_post` exchanges: assert Basic sends an `Authorization: Basic` header without form client credentials, and Post sends `client_id`/`client_secret` in the form without an Authorization header; then cover PKCE, missing/mismatched/replayed/expired/concurrent state, provider errors, malformed token responses, wrong nonce/issuer/audience/signature, expiration, and invalid `at_hash`, asserting every failure establishes no session and returns to `/login` with a stable error plus preserved encoded target or `return_to=%2F`
- [ ] 2.7 Add cache/rotation tests proving normal callbacks do not rediscover/refetch, non-signature failures do not refresh, and concurrent failures against stale JWKS generation N cause one fetch while waiters retry cached N+1 without another request
- [ ] 2.8 Implement atomic callback consumption, selected-method exchange, complete token verification, generation-aware one-time JWKS refresh, session rotation/token discard, and safe success/error redirects back through `/login`; verify all protocol tests pass

## 3. Sessions, public/protected routing, and logout

- [ ] 3.1 Add session tests first for private cookie attributes, opaque/tampered/unknown cookies, deterministic five-minute transactions, absolute non-sliding twelve-hour sessions, and process-restart invalidation through a replaceable store/key and clock
- [ ] 3.2 Implement private opaque `MemoryStore` sessions, locked one-time transaction registry, explicit expiry checks, and minimal authenticated records; verify no provider token is serialized anywhere
- [ ] 3.3 Add `ApiError::Unauthorized` and stable `AUTHENTICATION_REQUIRED` JSON mapping with unit tests; verify all existing API error mappings remain unchanged
- [ ] 3.4 Add router tests first for auth-disabled existing-route compatibility plus inert `{ enabled: false }` auth config, enabled public `/health`, `/login`, `/auth/config`, auth endpoints/non-HTML login assets, protected `/index.html`/nonexistent paths/application fallback, server navigation redirects carrying path/query only, JSON 401 for `/api` and `/api/*`, and `/apiary` handling
- [ ] 3.5 Split non-HTML exact embedded-asset serving from protected index fallback and compose the enabled public/protected routers; verify `/login` can load its Svelte JS/CSS while direct `/index.html` and protected application routes cannot load without a session
- [ ] 3.6 Add tests for public auth config returning only `{ enabled: false }` when disabled and only enabled/provider name when enabled, default/custom labels, enabled-only optional app-info flag, and unchanged disabled app-info shape; implement those response boundaries
- [ ] 3.7 Add logout tests first for authenticated/anonymous POST, backend revocation, cookie expiry, former-cookie rejection, and `303 /login?logged_out=1`; implement idempotent local logout without provider calls
- [ ] 3.8 Extend state/startup for optional retained OIDC runtime and add end-to-end cookie tests through login initiation, successful callback to the stored safe `return_to`, authenticated UI/API, every callback failure redirecting to `/login?error=...&return_to=...` (using `%2F` when no safe target remains), logout, and restart; verify no users/accounts/session table is introduced

## 4. Public Svelte login experience

- [ ] 4.1 Add route/layout tests first proving `/login` resolves auth config outside app navigation/pull-to-refresh, performs no protected hydration, renders only when enabled, and replace-navigates to `/` without auth controls when disabled
- [ ] 4.2 Add login-page component tests first for Gazel branding, concise authentication-required text, exactly one OIDC button, absence of username/password/registration controls, and public auth-config failure behavior
- [ ] 4.3 Add login state tests for default/custom provider label, encoded `return_to` propagation to `/auth/login`, generic/temporary failure alerts, unknown error sanitization, signed-out confirmation, retry button, and no automatic login
- [ ] 4.4 Implement `ui/src/routes/login/+page.svelte` and the root-layout public-route branch using existing Svelte 5/design-system patterns; verify all focused login/layout tests pass without an auth library

## 5. Frontend session lifecycle and settings logout

- [ ] 5.1 Add API-client tests first for exact `401 + AUTHENTICATION_REQUIRED` navigation to `/login`, `%23` fragment encoding inside `return_to`, one navigation under concurrent failures, nonmatching errors, and both direct export paths
- [ ] 5.2 Implement one shared API error-response handler used by JSON and export requests; send the top-level browser to `/login` exactly once on expiry while preserving normal typed errors otherwise
- [ ] 5.3 Add settings tests first for optional `AppInfo.auth_enabled`, hidden controls in disabled mode, translated enabled Authentication section, POST logout form, and `/login?logged_out=1` destination; implement and verify disabled settings remain unchanged
- [ ] 5.4 Add `/auth` to the Vite development proxy and matching English/German login/error/provider/logout keys; verify placeholder/key parity, formatting, linting, type checking, and focused vitest tests

## 6. User and deployment documentation

- [ ] 6.1 Update `README.md` with dedicated login-page behavior, callback registration, required auth settings plus optional provider name, Basic/Post interoperability, public/protected resources, server path/query versus SPA-hash return behavior, mandatory login-page callback errors, shared-data semantics, HTTPS/proxy guidance, restart/replica limits, and local-versus-provider logout
- [ ] 6.2 Update `docker-compose.yml` with a commented optional OIDC example including provider display name while keeping authentication disabled by default; verify `docker compose config` and ensure no real secret

## 7. Validation and review readiness

- [ ] 7.1 Run focused Rust auth/config/router tests and direct frontend tests; fix only change-related failures and record results
- [ ] 7.2 Run `cargo fmt -- --check`, `cargo clippy -- -D warnings`, `npm run format:check --prefix ui`, `npm run lint --prefix ui`, `npm run check --prefix ui`, and `cargo test`; verify the full pre-review gate passes
- [ ] 7.3 Run strict validation for this OpenSpec change, repository-wide validation, and implementation verification against every proposal/design/spec requirement; report unrelated pre-existing failures without editing them
- [ ] 7.4 Review the final diff for token/secret logging, browser storage, open redirects, forwarded-header trust, client-auth mismatch, JWKS refresh races, login hydration loops, accidental public application routes, local-user concepts, provider-specific behavior, and unrelated refactors
