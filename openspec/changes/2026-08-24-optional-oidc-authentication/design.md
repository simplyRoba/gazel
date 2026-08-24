## Context

See `proposal.md` for motivation. Gazel is one Axum process with a cloneable `AppState`, a single `/api` router, a root `/health` route, and an embedded SvelteKit SPA served by a fallback handler. Configuration is currently synchronous and tolerant, and the application has no identity model. The frontend is a same-origin static SPA, so it already sends cookies without an auth SDK.

The security contract is defined by the six delta specs in this change. The design must preserve the disabled path, fail closed before serving traffic when enabled, implement protocol validation through a maintained OIDC crate, and remain appropriate for Gazel’s small single-binary deployment model.

## Goals / Non-Goals

**Goals:**

- Make the enabled and disabled route graphs structurally obvious and auditable.
- Keep OIDC protocol parsing, JOSE cryptography, discovery, and claim verification inside a maintained standards library.
- Keep browser state to one encrypted opaque cookie and make local logout revocable on the server.
- Support HTTPS reverse proxies using one configured public origin, without any forwarded-header trust decision.
- Produce deterministic security tests against a local provider with no real IdP dependency.

**Non-Goals:**

- Long-lived provider access, refresh, or user-info access after login.
- Persistent login across Gazel process restarts or active/active multi-instance deployment.
- Provider-specific options, dynamic client registration, RP-initiated provider logout, bearer-token API authentication, or refresh-token handling.
- Any frontend identity profile, logout control, auth library, local user record, or authorization policy.

## Decisions

### 1. Build distinct disabled and enabled Axum route graphs

**Decision:** `server::router` will keep the current router shape when `AppState.auth` is `None`. When auth is enabled, it will compose:

1. exact public routes: `/health`, `GET /auth/login`, `GET /auth/callback`, and `POST /auth/logout`;
2. one protected router containing `/api/info`, the complete nested `/api` router, and the embedded SPA fallback;
3. one outer session layer needed by both public auth handlers and protected middleware;
4. the existing access log around the completed graph.

The authentication middleware will classify only `path == "/api"` or `path.starts_with("/api/")` as API requests. Missing/invalid sessions return `ApiError::Unauthorized("AUTHENTICATION_REQUIRED")`; other protected paths (including lookalikes such as `/apiary`) get a `303 See Other` to `/auth/login`.

**Rationale:** Applying one global auth layer would accidentally protect `/health` and the callback. Protecting only the API would expose the SPA. A protected sub-router makes both mistakes difficult and lets the disabled branch retain current behavior, including fallback handling for paths that only become auth endpoints in enabled mode.

**Alternatives:** Per-handler extractors were rejected because they are easy to omit. Frontend route guards were rejected because static assets and API endpoints still need a backend boundary.

### 2. Use `openidconnect` directly rather than an authentication framework

**Decision:** Add `openidconnect` 4.x with its default async `reqwest`/Rustls support and `timing-resistant-secret-traits`. Use `CoreProviderMetadata::discover_async`, `CoreClient`, Authorization Code flow, `PkceCodeChallenge::new_random_sha256`, generated `CsrfToken` and `Nonce`, token exchange with the stored verifier, and `id_token.claims(client.id_token_verifier(), nonce)`.

Wrap a `reqwest::Client` configured with `redirect::Policy::none()` in a small `AsyncHttpClient` transport guard. Before delegating each discovery, associated JWKS, or token request, the guard accepts HTTPS or an HTTP loopback URL only. This is necessary because `discover_async` retrieves the associated JWKS before application code can inspect the resulting metadata. After discovery, validate the authorization, token, and JWKS endpoint URLs again, require Authorization Code support, and require at least one JWKS signature-verification key compatible with an advertised ID-token signing algorithm.

Startup discovery and its associated JWKS retrieval must succeed before the listener binds. Callback processing will use freshly discovered metadata/JWKS so a normal provider signing-key rotation does not require a Gazel restart. Discovery and token calls are single attempts: startup failure aborts; a runtime provider/network failure returns a generic callback error and requires a new login attempt.

In addition to the verifier’s signature, issuer, audience, expiry, and nonce checks, callback code will verify `at_hash` whenever the ID token contains it. The default verifier behavior that rejects untrusted additional audiences will remain enabled. No unsafe verifier relaxation will be used.

**Rationale:** `openidconnect` owns the protocol types, discovery rules, OAuth exchange, JWKS/JWS algorithms, and claim verification. A small set of Axum handlers around it is less surface area than a generalized auth framework. Enabling timing-resistant secret equality provides an appropriate state comparison without custom cryptography.

**Alternatives:** Raw `oauth2` lacks OIDC ID-token/discovery validation. Provider-specific crates violate portability. Axum OIDC wrappers either pin older protocol dependencies or add identity/authorization abstractions Gazel does not need. Manual JWT/PKCE/nonce cryptography is explicitly rejected.

### 3. Parse auth configuration into a typed optional boundary

**Decision:** Change `Config::load`/`load_from` to return `Result<Config, ConfigError>`. Existing port/database/log-level fallback behavior remains. An absent enable flag maps to `None`; only `true` creates `Some(AuthConfig)`. A malformed enable flag fails rather than silently disabling security. Explicit `false` ignores all auth-only values.

`AuthConfig` contains parsed external and issuer URLs, client credentials, and decoded session-key bytes. The secret is standard Base64 that must decode to at least 64 bytes, suitable for `cookie::Key::try_from`. URLs must be absolute, credential/query/fragment-free, and HTTPS except for loopback HTTP. Gazel is root-mounted today, so a non-root external path is rejected instead of pretending subpath support. The external URL is normalized to its origin, and the callback is built by joining the fixed `/auth/callback` path to that origin; request headers never participate.

**Rationale:** A typed `Option<AuthConfig>` makes disabled behavior explicit and prevents partial enabled configurations. Validating before network I/O gives actionable startup errors. Base64 preserves full random key entropy and avoids interpreting an operator secret as ad hoc key material.

**Alternatives:** Tolerant defaulting is unsafe for an enable flag. Deriving the origin from `Host` or `X-Forwarded-*` enables host-header/proxy-confusion attacks. Adding trusted-proxy configuration would be more complex than the explicit public URL Gazel already needs for OIDC registration.

### 4. Use encrypted opaque sessions backed by `tower-sessions::MemoryStore`

**Decision:** Add `tower-sessions` 0.15 with the `private` feature. The `gazel_session` cookie contains only a random session identifier protected with authenticated encryption. Configure `HttpOnly`, `SameSite=Lax`, `Path=/`, no `Domain`, and `Secure=true` except for validated loopback HTTP development. Successful callback calls `cycle_id()` before writing the authenticated marker. Logout calls `flush()` so the backend record and cookie are both invalidated.

A process-local `Mutex<HashMap<state, LoginTransaction>>` holds nonce, PKCE verifier, and an absolute five-minute `expires_at`. The tower session stores only the state that binds its browser to that registry entry. Callback first compares the returned and session-bound state, then removes the registry entry while holding the mutex and releases the lock before any provider await. Exactly one concurrent callback can therefore acquire a transaction. Expired entries are pruned when login/callback touches the registry.

After verification, the transaction binding is replaced with a minimal authenticated record containing the OIDC subject, login time, and absolute `expires_at` twelve hours after callback; access, refresh, and ID tokens are immediately dropped. Middleware compares that timestamp on every protected request and never extends it. Tower session expiry is set to the same absolute time as defense in depth. A small clock abstraction uses wall-clock UTC in production and an injected deterministic clock in tests.

The session store and atomic transaction registry are intentionally process-local. Restarting Gazel invalidates every session and pending login, and multiple instances would require sticky routing; both are documented. This is acceptable for Gazel’s current single-process/single-binary deployment and keeps auth state out of the application schema.

**Rationale:** Server storage gives real local revocation and keeps sensitive transaction values out of the cookie. Authenticated encryption adds defense in depth for the opaque identifier. `SameSite=Lax` permits the cross-site top-level OIDC callback while the state/nonce transaction protects the flow. A POST-only logout avoids cross-site top-level GET logout.

**Alternatives:** A self-contained encrypted auth cookie cannot revoke a copied pre-logout cookie without server state. A custom map/cookie implementation would duplicate lifecycle and cookie security code. The SQLx session-store release currently targets a different SQLx/core version than Gazel and would add a session table plus cleanup task; Redis is disproportionate. Persistent sessions can be reconsidered if Gazel later supports multi-instance operation.

### 5. Consume login transactions before provider exchange

**Decision:** Login first reads the authenticated record. A valid authenticated session is redirected to `/` unchanged, preventing `GET /auth/login` from becoming a forced-logout endpoint. For an unauthenticated/expired session, login removes any prior pending registry entry, creates one new transaction, stores its state binding in the tower session, and redirects to the provider.

Callback removes the session binding, performs timing-resistant comparison against returned state, and atomically removes the matching registry transaction before any provider network await. It then checks the absolute transaction expiry, reconstructs the nonce and PKCE verifier, performs fresh discovery and token exchange, verifies the ID token and optional access-token hash, rotates the session ID, and writes the authenticated record with absolute twelve-hour expiry.

All callback failure paths leave no authenticated record and require a fresh login. Validation/authorization failures return a generic `400 Bad Request`; unavailable or malformed backend provider responses return a generic `502 Bad Gateway`; session infrastructure failures return `500 Internal Server Error`. Responses and logs never include codes, state, nonce, PKCE values, client secrets, provider tokens, callback query strings, or provider error descriptions.

**Rationale:** Removing the transaction first makes callbacks one-use even when exchange or verification fails. It also avoids retaining attacker-controlled callback attempts for replay. The user cost of a fresh login after a transient error is small and avoids retry/replay ambiguity.

**Alternatives:** Keeping a transaction after token failure improves retry convenience but risks code replay and creates more state transitions. Multiple concurrent login transactions per browser are unnecessary for Gazel.

### 6. Keep identity and frontend behavior minimal

**Decision:** The verified `sub` value is not joined to application data and is not exposed through a new API. Every valid subject receives the same application access. The frontend receives no token and needs no auth library or local-storage change. Normal browser navigation is redirected by Axum; `fetch` calls continue to receive and surface the existing typed API error shape with a new stable 401 code. Add `/auth` to the Vite development proxy only.

**Rationale:** The backend is the security boundary, and the existing same-origin browser automatically carries the HTTP-only cookie. Avoiding an identity endpoint, route store, or frontend guard keeps this an authentication-only change and preserves disabled UI behavior byte-for-byte apart from the rebuilt bundle metadata.

**Alternatives:** Adding a user-info endpoint and logout UI would enlarge the data contract and layout scope without being required to protect Gazel. Automatic frontend redirect on every 401 could turn background API failures into navigation loops; direct browser navigation already has the intended login behavior.

### 7. Test through a local standards-shaped provider

**Decision:** Add an integration-test mock provider using an ephemeral local Axum listener. It will expose discovery, authorization, token, and JWKS endpoints. Test ID tokens and JWKS will be built and signed with `openidconnect`’s provider-side RSA types and a test-only key fixture, not custom JWT cryptography. The provider will record PKCE and allow controlled wrong nonce/audience/issuer/signature, expired token, OAuth error, malformed/empty/unusable JWKS, insecure endpoint metadata, malformed discovery, and malformed token responses.

Router tests will manually preserve `Set-Cookie` values across `oneshot` requests and drive the provider over loopback HTTP. A synchronized token-call counter and concurrent callbacks prove one-time atomic transaction consumption. Focused configuration and session unit tests use the deterministic clock to cover disabled defaults, every invalid configuration class, opaque/tampered/expired sessions, absolute non-sliding expiry, and cookie policy.

**Rationale:** This exercises the actual discovery, HTTP, token, and JOSE path without an external IdP or a mock of the code under test. Unit tests remain preferable for combinatorial validation and expiry boundaries.

## Risks / Trade-offs

- **[In-memory sessions log users out on restart and do not support non-sticky replicas]** → Document the limitation; fail closed with unknown cookies; revisit a compatible SQLite store only when deployment needs justify it.
- **[OIDC provider downtime prevents startup when auth is enabled]** → This is deliberate fail-closed behavior. Container restart policy and provider recovery provide retry; Gazel never silently disables auth.
- **[Fresh discovery during callback adds low-frequency discovery and JWKS round trips]** → Login volume is tiny, and current keys/metadata improve key-rotation interoperability. The guarded HTTP client rejects redirects and insecure non-loopback endpoints; errors are bounded to the login attempt.
- **[A stolen session cookie is sufficient until expiry or logout]** → Require TLS outside loopback, use `Secure`/`HttpOnly`/authenticated encryption, rotate after login, keep the lifetime at twelve hours, and support immediate server-side deletion.
- **[SameSite=Lax permits cookies on cross-site top-level GET callbacks]** → This is necessary for standard OIDC redirects; one-time state, nonce, and PKCE bind the callback.
- **[All authenticated identities share all data]** → This is an explicit product constraint, prominently documented; authentication is not tenant isolation.
- **[Existing unrelated OpenSpec change fails global validation]** → Validate this change strictly and report the pre-existing failure without modifying unrelated artifacts.

## Migration Plan

1. Deploy with `GAZEL_AUTH_ENABLED` absent or `false`; runtime behavior remains unchanged and rollback is simply the previous image.
2. Register `<GAZEL_EXTERNAL_URL>/auth/callback` as an Authorization Code callback at the generic OIDC provider.
3. Generate a 64-byte random Base64 auth secret, provide all six settings, and ensure the external/issuer URLs use HTTPS outside loopback.
4. Restart Gazel. Local validation and provider discovery complete before the listener starts; any error leaves the service unavailable rather than exposed.
5. Verify public `/health`, browser login, protected `/api`, and local logout through the deployment proxy.
6. To disable or roll back, set `GAZEL_AUTH_ENABLED=false` or deploy the prior image. Existing application data is untouched because no schema migration is introduced.
