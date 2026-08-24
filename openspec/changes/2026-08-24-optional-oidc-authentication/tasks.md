## 1. Dependencies and configuration

- [ ] 1.1 Add maintained `openidconnect`, `tower-sessions` private-cookie support, and only the small direct utility dependencies required by the design; verify dependency resolution and a backend-only `cargo check` succeed
- [ ] 1.2 Add configuration tests first for disabled defaults, explicit false, malformed enable flags, missing/empty enabled values, Base64 secret strength, HTTPS/loopback URL policy, forbidden URL components/paths, and exact callback construction; verify the focused config tests fail before implementation and pass afterward
- [ ] 1.3 Implement fallible typed `AuthConfig` loading while preserving all legacy port/database/log-level fallback behavior; verify all existing and new config tests pass

## 2. OIDC runtime and secure sessions

- [ ] 2.1 Add a local test OIDC provider with discovery, authorization, token, and JWKS endpoints plus crate-signed ID-token fixtures and controllable malformed/invalid responses; verify it runs entirely on loopback without a real IdP
- [ ] 2.2 Add protocol tests first for successful and malformed startup discovery/JWKS retrieval, empty or unusable key sets, insecure discovered endpoints, backend redirect refusal, login parameters, already-authenticated login behavior, fresh state/nonce/PKCE, exact external callback URL despite forwarded headers, and token-endpoint PKCE use; verify the focused auth tests expose missing implementation
- [ ] 2.3 Implement OIDC discovery/client initialization with a transport-validating no-redirect Rustls HTTP client, associated JWKS usability checks, and fresh callback discovery; verify valid discovery succeeds and unreachable, redirected, insecure-endpoint, wrong-issuer, incomplete, empty-key, unusable-key, or malformed discovery fails closed
- [ ] 2.4 Add callback tests first for valid exchange plus missing/mismatched/replayed/expired state, concurrent replay, provider authorization errors, missing/malformed token responses, wrong nonce/issuer/audience/signature, expired ID tokens, and invalid access-token hashes; verify no failure establishes a session and at most one concurrent callback reaches token exchange
- [ ] 2.5 Implement login and callback handlers with an atomically consumed in-process transaction registry bound to the browser session, timing-resistant state comparison, nonce, S256 PKCE, complete ID-token verification, optional `at_hash` verification, generic safe errors, session-ID rotation, and immediate token discard; verify all callback and login tests pass
- [ ] 2.6 Add session tests first for cookie attributes, opaque/tampered/unknown cookies, deterministic five-minute transaction expiry, absolute non-sliding twelve-hour authenticated expiry, and session invalidation after restart/store replacement; verify the expected security boundaries through an injectable clock seam
- [ ] 2.7 Implement authenticated-encrypted opaque `tower-sessions::MemoryStore` sessions with explicit absolute transaction/auth `expires_at` checks, a testable clock, and minimal authenticated records; verify session tests pass and no provider token is serialized into cookie/session data
- [ ] 2.8 Add logout tests first for authenticated and anonymous requests, expired-cookie response behavior, and rejection of the former cookie; implement idempotent public `POST /auth/logout` using backend session flush and verify the tests pass

## 3. Router and API boundary

- [ ] 3.1 Add `ApiError::Unauthorized` and its stable `AUTHENTICATION_REQUIRED` message with unit tests for `401` JSON mapping; verify all existing API error mappings remain unchanged
- [ ] 3.2 Add router tests first proving auth-disabled UI/API compatibility, public `/health`, public auth endpoints, unauthenticated UI redirect, unauthenticated `/api` and `/api/*` JSON 401 without `Location`, non-API handling for lookalike paths such as `/apiary`, and protected unknown/static paths; verify they fail before route composition
- [ ] 3.3 Extend application state/startup for optional discovered auth state and compose separate public/protected enabled routers while preserving the original disabled graph; verify the router boundary tests pass
- [ ] 3.4 Add end-to-end tests carrying real session cookies through login, mock-provider authorization/callback, authenticated UI/API access, invalid sessions, expiry, replay, and logout; verify the complete flow passes without frontend token access or a users/accounts table

## 4. Development and deployment documentation

- [ ] 4.1 Add `/auth` to the Vite development proxy and add matching English/German `error.AUTHENTICATION_REQUIRED` translations; verify translation completeness tests, frontend formatting, linting, and type checking pass without adding an auth library or changing disabled UI behavior
- [ ] 4.2 Update `README.md` with optional OIDC setup, callback registration, all six `GAZEL_*` settings, secret generation, public/protected route behavior, shared-data/no-authorization semantics, HTTPS and reverse-proxy guidance, session restart/replica limitations, and local logout; verify examples contain no real secrets
- [ ] 4.3 Update `docker-compose.yml` with a commented/optional OIDC configuration example while keeping authentication disabled in the runnable default; verify `docker compose config` succeeds

## 5. Validation and review readiness

- [ ] 5.1 Run focused Rust auth/config/router tests and the direct frontend test suite; fix only change-related failures and record the results
- [ ] 5.2 Run `cargo fmt -- --check`, `cargo clippy -- -D warnings`, `npm run format:check --prefix ui`, `npm run lint --prefix ui`, `npm run check --prefix ui`, and `cargo test`; verify the complete pre-review gate passes
- [ ] 5.3 Run strict OpenSpec validation for `2026-08-24-optional-oidc-authentication`, run repository-wide OpenSpec validation, and verify implementation against every proposal/design/spec requirement; report any unrelated pre-existing validation failure without modifying that change
- [ ] 5.4 Review the final diff for secret/token logging, browser storage, open redirects, forwarded-header trust, accidental public routes, users/accounts migrations, provider-specific behavior, and unrelated refactors; verify none are present before requesting human review
