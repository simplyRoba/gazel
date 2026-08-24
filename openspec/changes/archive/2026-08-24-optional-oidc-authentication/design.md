## Context

See `proposal.md` for motivation. Gazel is one Axum process with a cloneable `AppState`, a single `/api` router, a root `/health` route, and an embedded client-only SvelteKit SPA served by one fallback handler. The root Svelte layout currently renders the full app shell and starts protected settings/vehicle hydration for every route. The same-origin frontend already sends cookies, but its API helper only throws on errors; settings initialization swallows failures and export functions use separate direct `fetch()` paths.

The security contract is defined by the ten delta specs in this change. The design must preserve existing disabled application/API behavior apart from the new inert public auth-config endpoint, fail closed before serving enabled traffic, provide a public branded login route without triggering protected hydration, recover an already-open SPA when its session expires, and remain appropriate for Gazel’s small single-binary deployment model.

## Goals / Non-Goals

**Goals:**

- Make enabled public/protected route boundaries structurally obvious and auditable.
- Provide one minimal public Svelte login experience using Gazel’s existing branding/design system.
- Keep OIDC protocol parsing, JOSE cryptography, discovery, and claim verification inside a maintained standards library.
- Interoperate with confidential clients using standard Basic or request-body token authentication.
- Keep browser state to one encrypted opaque cookie and make local logout revocable on the server.
- Restore the user’s safe local page after initial login or expiry reauthentication.
- Support reverse proxies through a configured public origin without forwarded-header trust.
- Test against a local provider with no real IdP dependency.

**Non-Goals:**

- Long-lived provider access, refresh, or user-info access after login.
- Persistent login across process restarts or active/active multi-instance deployment.
- Provider-specific integration, dynamic registration, provider logout, bearer API auth, or refresh tokens.
- Identity profiles, usernames, passwords, registration, local users, authorization policy, per-user ownership, or a frontend auth framework.

## Decisions

### 1. Compose public login resources separately from protected application routes

**Decision:** Disabled mode retains current application/API/static/fallback behavior and adds only public `GET /auth/config` returning `{ enabled: false }`, allowing the compiled login route to self-disable. Enabled mode composes:

1. public `GET /login`, returning `303 /` when a valid Gazel session is present and otherwise returning embedded `index.html` so SvelteKit renders the dedicated route;
2. public exact non-HTML static assets needed by both login and application bundles (`/_app/*`, logo/manifest/favicon resources), explicitly excluding `index.html`;
3. public `/health`, `GET /auth/config`, `GET /auth/login`, `GET /auth/callback`, and `POST /auth/logout`;
4. one protected router containing exact `/api`, all `/api/*` routes (including `/api/info`), and every application SPA document/fallback other than `/login`;
5. one outer session layer and the existing access log.

The embedded handler will be split conceptually into non-HTML exact-asset serving and index fallback so public assets do not make every unknown SPA route public. `index.html` is served publicly only for exact `/login` without a valid session; a valid session receives `303 /`. Direct `/index.html`, nonexistent asset paths, and all other document fallbacks pass through the protected boundary. Public assets contain code and branding only, never application records or OIDC tokens.

The middleware classifies only `path == "/api"` or `path.starts_with("/api/")` as API requests. Missing/invalid API sessions return JSON `ApiError::Unauthorized("AUTHENTICATION_REQUIRED")`. Unauthenticated document navigation receives `303` to `/login?return_to=<encoded request path and query>`. The backend uses only `Request::uri().path_and_query()`; browser fragments never reach it. Asset/subresource requests remain limited to exact public assets and are never treated as a return destination.

**Rationale:** A Svelte login page cannot run unless its JS/CSS assets are public. Splitting exact assets from protected fallback exposes no user data while preserving the application boundary. A global auth layer would protect login/health/callback; API-only auth would expose application routes.

**Alternatives:** A backend-rendered login page would duplicate Svelte branding/styles. Making all SPA fallback public would expose protected application documents and trigger API churn. Per-handler guards are easy to omit.

### 2. Use `openidconnect` directly and apply auth-method metadata explicitly

**Decision:** Add `openidconnect` 4.x with async `reqwest`/Rustls and `timing-resistant-secret-traits`. Use startup `CoreProviderMetadata::discover_async`, `CoreClient`, generated state/nonce, S256 PKCE, code exchange, and the normal ID-token verifier.

Wrap `reqwest::Client` using `redirect::Policy::none()` in an `AsyncHttpClient` transport guard accepting HTTPS or HTTP loopback only. Validate exact issuer, authorization/token/JWKS endpoint transport, Authorization Code support, and usable signing metadata/JWKS before binding the listener.

`CoreClient::from_provider_metadata` 4.0.1 ignores `token_endpoint_auth_methods_supported` (upstream issue #215), so client construction explicitly selects:

- omitted metadata → `AuthType::BasicAuth`;
- Basic listed (alone or with Post) → `AuthType::BasicAuth`;
- Basic absent and Post listed → `AuthType::RequestBody`;
- neither → startup error.

The selected method is retained and reapplied whenever cached verification state is rebuilt.

Startup endpoints and JWKS are cached; callbacks do not rediscover. Cache state includes a monotonically increasing JWKS generation and an optional process-monotonic failed-refresh retry deadline scoped to the current generation. On an eligible no-matching-key/signature failure, the callback records the generation it used and acquires a refresh mutex. Under the lock it first checks the current generation: if another callback already advanced it, retry with those keys and perform no fetch; if the same generation is inside its failure cooldown, return `provider_unavailable` without fetching; otherwise fetch only the configured JWKS.

A successful targeted fetch replaces the cached verification keys, increments the generation, clears the failure deadline, and permits one ID-token verification retry. A failed or unusable JWKS response retains the last usable client and generation, records a 30-second cooldown, and returns `provider_unavailable`; concurrent waiters and later attempts during that window perform no additional JWKS request. Once the cooldown expires, a later login may make one new targeted fetch, allowing provider recovery without a Gazel restart. Claim failures other than key/signature never refresh. Token exchange and full discovery are never retried.

Normal verification checks signature, issuer, audience, expiry, nonce, and default additional-audience policy. Gazel also verifies `at_hash` when present. No unsafe verifier relaxation is allowed.

**Rationale:** The crate owns protocol and cryptography, while Gazel fills one documented interoperability gap. Generation-aware single-flight refresh prevents a stale-key thundering herd; the failure cooldown bounds provider load without turning one transient failure into a process-lifetime tombstone.

**Alternatives:** Raw OAuth lacks OIDC claims verification. Full callback discovery is excessive. A mutex without generation comparison serializes but does not deduplicate waiting refreshes. Permanently suppressing refresh for a failed generation prevents automatic recovery and is rejected.

### 3. Parse enabled auth into five functional settings plus optional display metadata

**Decision:** `Config::load`/`load_from` becomes fallible while legacy port/database/log defaults remain. Absent enable means disabled; malformed enable fails rather than silently exposing the app; explicit false ignores auth-only values.

Enabled operation uses five functional settings: `GAZEL_AUTH_ENABLED=true`, external URL, issuer, client ID, and client secret. `GAZEL_OIDC_PROVIDER_NAME` is optional, trimmed, limited to 80 non-control Unicode scalar values, and defaults to `OpenID Connect`. It is display-only and never affects issuer/client decisions.

URLs must be absolute, credential/query/fragment-free, and HTTPS except loopback HTTP. Gazel rejects an external non-root path, normalizes the origin, and joins `/auth/callback`. Request headers never participate.

No operator-managed cookie secret setting is introduced. Enabled startup securely generates one private-cookie key with a fallible OS-random API. Generation failure aborts startup. A persistent operator key could not preserve sessions after `MemoryStore` restart and would add needless setup.

**Rationale:** The enable flag and four required OIDC values form the five-setting functional boundary; the optional label adds display metadata only. No extra cryptographic secret lifecycle is needed, and typed optional config prevents partial enabled operation.

### 4. Use encrypted opaque sessions plus an atomic login registry

**Decision:** Use `tower-sessions` 0.15 private cookies with `MemoryStore`. The `gazel_session` cookie contains only a random ID under authenticated encryption. Set `HttpOnly`, `SameSite=Lax`, `Path=/`, no Domain, and Secure except validated loopback HTTP.

A process-local locked map keyed by state stores nonce, PKCE verifier, validated return target, and absolute five-minute expiry. The tower session stores the state binding. Callback compares returned/session state, atomically removes the registry entry while locked, then releases before provider I/O. Exactly one callback can acquire it. Expired entries are pruned during registry operations; missing, expired, replayed, or concurrently rejected callbacks return to the generic `/login` authentication-failed state rather than a dead-end response.

After verification, write only subject/login time/absolute twelve-hour expiry and discard tokens. Middleware checks expiry without sliding it; tower session expiry mirrors it. Use an injectable clock in tests. Successful callback cycles the ID. Logout flushes. Restart replaces store/key and invalidates all sessions. Multi-instance operation requires sticky routing and remains unsupported/documented.

**Rationale:** Server state enables real local revocation. Private cookies add defense in depth. The locked registry fixes request-scoped session-store callback races.

### 5. Carry only encoded, validated local return targets

**Decision:** Server middleware and the SPA each encode the complete target available to them as one `return_to` query value, but their inputs differ:

- middleware sees only the HTTP request path and query. A request target `/settings?tab=data` becomes `/login?return_to=%2Fsettings%3Ftab%3Ddata`; the backend cannot receive, infer, or preserve a browser fragment from that navigation;
- the already-running SPA can read `location.pathname + location.search + location.hash`. For `/settings?tab=data#export`, it produces `/login?return_to=%2Fsettings%3Ftab%3Ddata%23export`, carrying `%23export` as query-parameter data rather than as `/login`'s own fragment.

The login Svelte page reads the decoded query value and uses `URLSearchParams` to build `/auth/login?return_to=...`. Backend `/auth/login` is authoritative: after percent-decoding the query parameter, it requires a single-leading-slash same-origin protected UI target, limits the serialized target to 2,048 UTF-8 bytes, and rejects authority/scheme, backslash, control characters, and `/api`, `/auth`, `/health`, or `/login` targets. Middleware applies the same limit before constructing its login redirect. Invalid, absent, or oversized input becomes `/` before any `Location` serialization or transaction storage.

The backend transaction stores the validated serialized target. Successful callback redirects to that stored `return_to`. It can contain a hash suffix only when the SPA previously supplied that suffix as percent-encoded query-parameter data; no HTTP request fragment was received by the backend. A valid authenticated session visiting `/auth/login` returns directly to the safe target without starting OIDC or destroying its session.

**Rationale:** This restores all context available to the component that initiated reauthentication while preventing open redirects and false claims about server-visible fragments. Backend validation means a tampered login-page query cannot weaken safety.

### 6. Add a standalone public Svelte login surface

**Decision:** Add `ui/src/routes/login/+page.svelte`. The root layout branches on pathname before protected initialization:

- `/login` without a valid backend session: render only the child login page, no app navigation/fill-up controls/pull-to-refresh, and do not initialize settings, vehicles, fill-ups, stats, or other protected stores;
- `/login` with a valid backend session: the server returns `303 /`, so the authentication-required page never renders;
- all application routes: retain the current app shell/hydration.

The page uses the existing Logo/design tokens and translations. It displays Gazel branding, short authentication-required text, and exactly one `Continue with {provider}` link/button. There are no local credential controls.

On mount it fetches public `GET /auth/config`. Enabled response contains only `{ enabled: true, provider_name }`; disabled response contains only `{ enabled: false }` and causes a replace-navigation to `/` before auth controls render. The endpoint never exposes issuer details, client ID/secret, discovered endpoints, session state, or tokens. A failed/malformed response shows a generic usable unavailable state, not credential fallback.

Stable query states are `error=authentication_failed`, `error=provider_unavailable`, and `logged_out=1`. Unknown error values map to the generic failure message and are never rendered directly. Every state retains the explicit OIDC button; none auto-starts login.

**Rationale:** A dedicated page gives enabled users a deliberate, branded boundary and makes failures/logout understandable. The inert disabled status prevents the compiled route from introducing a disabled-mode auth UI. Skipping root hydration prevents `/login` from calling protected APIs and looping on 401.

### 7. Return callback failures and logout to the login page

**Decision:** Callback first validates/consumes state, retaining the safe target locally when one is available. Every callback failure returns `303` to `/login?error=<stable-error>&return_to=<encoded-safe-target>`. State, provider-denial, protocol, token, claim, replay, expiry, concurrency, and other authentication failures use `authentication_failed`; provider token/JWKS communication failures use `provider_unavailable`. The `return_to` parameter is always present and uses `%2F` when no validated transaction target remains. Failed callbacks never redirect directly to `/` and never expose or log provider descriptions, code, state, nonce, PKCE values, tokens, or raw callback queries.

`POST /auth/logout` flushes the local session/cookie and returns `303 /login?logged_out=1`, even if already anonymous. It never depends on or automatically invokes provider logout/login.

**Rationale:** Users receive a retryable state rather than a dead callback response. Logout visibly ends the local session without immediately recreating it via provider SSO.

### 8. Recover an expired SPA and expose logout conditionally

**Decision:** One frontend error-response helper is used by `request()`, `exportAll()`, and `exportVehicle()`. On exactly 401 plus `AUTHENTICATION_REQUIRED`, a module guard navigates once to `/login?return_to=<encoded current pathname+search+hash>`. Every other response follows normal typed-error behavior.

This is session lifecycle handling only: no token, identity state, or auth framework enters the SPA.

Enabled authenticated `/api/info` adds `auth_enabled: true`; disabled mode omits it and preserves current shape. `AppInfo.auth_enabled` is optional. Settings conditionally renders a translated Authentication section with a normal form posting to `/auth/logout`.

**Rationale:** A static SPA may remain open past absolute expiry and otherwise becomes permanently erroring. Optional app info makes logout usable without changing disabled UI or exposing identity.

### 9. Test through a local standards-shaped provider

**Decision:** Add an ephemeral Axum provider with discovery, authorization, token, and JWKS endpoints. Sign fixtures with `openidconnect` provider-side RSA types. Modes cover Basic/Post/omitted/unsupported auth metadata, including wire assertions that Basic uses only the Authorization header and Post uses only request-body client credentials, plus PKCE, invalid claims/signatures, malformed responses, endpoint redirects/transport, provider errors, availability failures, and key rotation.

Counters prove normal callbacks do not rediscover/refetch. Concurrent stale-generation callbacks must cause one JWKS request and each retry at most once. A failed-refresh recovery test rotates the signing key, fails the first targeted JWKS request, proves no additional fetch occurs during the 30-second cooldown, restores the provider, and proves a later login refreshes and succeeds without restarting Gazel. Cookie tests cover atomic callbacks, expiry, tampering, restart, and logout.

Backend router coverage proves server navigation preserves path/query only, oversized targets default to `/`, and authenticated `/login` returns `303 /`. Vitest/component coverage includes standalone unauthenticated `/login` shell, no protected hydration, branding/text/one button, provider default/custom labels, SPA-encoded `location.hash`, safe error states, exact 401 handling and exports, navigation guard, optional app info, and logout form.

## Risks / Trade-offs

- **[Public static bundles reveal client code]** → Expected for a browser app; bundles contain no application records, secrets, or tokens, while every data/API route remains protected.
- **[In-memory sessions end on restart and do not support non-sticky replicas]** → Document; old cookies fail closed. Revisit SQLite only if deployment needs change.
- **[Provider downtime prevents enabled startup]** → Deliberate fail-closed behavior; deployment restart policy supplies retry.
- **[One signature failure can initiate key refresh]** → Requires a valid one-time transaction/token exchange; generation locking deduplicates concurrent failures, and a 30-second failure cooldown bounds repeated provider load while preserving later recovery.
- **[A stolen cookie works until expiry/logout]** → TLS, Secure/HttpOnly/private cookie, login rotation, absolute twelve-hour expiry, and server deletion reduce exposure.
- **[SameSite=Lax permits top-level callback]** → Necessary; state, nonce, and PKCE bind it.
- **[Provider SSO remains after local logout]** → Login page shows signed-out state and waits for explicit action; Gazel does not claim provider logout.
- **[All authenticated identities share all data]** → Explicit product constraint; authentication is not tenancy.
- **[Existing unrelated OpenSpec change fails global validation]** → Validate this change strictly and report the pre-existing issue without editing it.

## Migration Plan

1. Deploy with auth absent/false; existing application/API behavior and app-info shape remain unchanged, with only inert public `GET /auth/config` added.
2. Register `<GAZEL_EXTERNAL_URL>/auth/callback` at the provider.
3. Provide enable flag, external URL, issuer, client ID/secret, and optionally provider display name; no Gazel cookie secret is needed.
4. Restart. Secure key generation, discovery/JWKS retrieval, endpoint validation, and token-auth selection finish before listening; failure leaves the service unavailable, never exposed.
5. Verify public `/login` and assets, custom/default provider label, safe return path, callback error state, public health, protected API, expiry reauthentication, and settings logout through the proxy.
6. Disable or roll back without data migration; process-local sessions simply cease to exist.
