# Design

## Context

See `proposal.md` for motivation and `specs/` for observable changes. Gazel already separates `Authentication`, the provider runtime, browser flow, and session backend. Flowl concentrates protocol and transaction state in `auth/mod.rs`, but has stronger capacity, persistence, claims, and cache guarantees. Both use `openidconnect` 4.0.1, `tower-sessions` 0.15, Axum, and a static SvelteKit UI.

The paired change is `flowl/openspec/changes/unify-oidc-authentication/`. Both applications must continue to build from their own checkout and Docker context. Existing canonical capabilities remain authoritative except where these deltas explicitly change them.

## Goals / Non-Goals

**Goals:**

- Keep common protocol, navigation, transaction, and session code identical, with named app-policy inputs rather than independent implementations.
- Separate behavior changes from module moves, and verify each security boundary with an observable test.
- Preserve independently deployable apps, existing authentication-disabled behavior, and supported deployment policies.

**Non-Goals:**

- A shared crate, cross-repository runtime dependency, new auth framework, database schema, or distributed session store.
- Optional provider auto-redirect, UI restyling, provider logout, user identity storage, or authorization policy.
- New operator-facing timeout, response-size, or rate-limit settings. Resource limits need separately specified values and failure behavior; no claim is made that dependency defaults are unbounded.

## Decisions

### 1. Mirror one core and isolate explicit app adapters

Use the same `src/auth/{mod,config,runtime,transaction,session,routes,return_to}.rs` in both repositories. `mod.rs` is a small `Authentication` facade; `runtime.rs` owns provider protocol and guarded transport; `transaction.rs` owns browser-bound attempt coordination; `session.rs` owns the private-cookie layer and authenticated state. Other modules own validated config, protocol routes, and `SafeReturnTo` respectively.

Put app-specific values in `src/auth/app.rs`, excluded from source parity. It supplies environment prefix, cookie name, URL transport policy, and non-document return prefixes. Server adapters own application error conversion, protected route classification, index/static serving, and `/api/info` integration. Common modules must not import application database, MQTT, upload, or UI state.

Retain existing `openidconnect` and `tower-sessions` features; no hand-written JOSE or secret-comparison primitive. Convert state to the library's timing-resistant secret type before comparison. Store only local authentication timing/state, not the subject returned by token verification. Use one cohesive authenticated record rather than parallel independently meaningful session keys.

**Alternative:** a crate hosted in one repository eliminates source duplication but adds packaging/release coupling. That is deferred. Merely adopting similar filenames without exact core parity is insufficient.

### 2. Make synchronization a concrete review gate

Gazel is the initial landing reference for common source, not a runtime dependency. Mirror the listed Rust files, pure frontend `ui/src/lib/auth-core.ts`, and JSON vectors under `tests/fixtures/auth/` byte-for-byte. App-specific adapters, local-provider harnesses, branding, and navigation side effects remain outside that list.

Add an identical `scripts/check-auth-parity.mjs` and parity manifest in both repositories during implementation. The manifest enumerates common files; the script requires an explicit `--peer <checkout>` and fails for missing peers/files or any differing common file. It must not silently succeed when comparison is unavailable. Run it against both coordinated branch checkouts before review and record the compared revisions. Ordinary builds/tests and Docker images never require the peer checkout or download its source.

Common-core changes must update the peer change in the same review cycle. JSON return-target and callback vectors run locally in each app; Rust and TypeScript consume the same return vectors. This makes divergent behavior detectable even where application test harnesses differ.

**Alternative:** a sibling Cargo path dependency breaks standalone builds. Unchecked copy/paste leaves the original maintenance problem unresolved.

### 3. Keep deployment policy explicit, not accidentally identical

| Input | Gazel | Flowl |
|---|---|---|
| Environment prefix | `GAZEL_` | `FLOWL_` |
| Cookie name | `gazel_session` | `flowl.sid` |
| Production URL policy | HTTPS or HTTP on `localhost`, `127.0.0.1`, `::1` | HTTPS only; HTTP fixtures test-only |
| Non-document return roots | `/api`, `/health`, plus common `/auth`, `/login` | Same plus `/uploads` |
| Integration | Existing `/api/info` enablement signal | Upload guards and PWA/logout integration |

The common config parser reads the enable flag fallibly, rejects present non-Unicode values, and reads other auth settings only when enabled. Preserve lenient parsing of unrelated settings. Preserve raw issuer spelling separately from parsed URL validation. Provider labels use Gazel's trimmed, non-control, at-most-80-Unicode-scalar rule in both apps.

`GET /auth/config` uses a strict discriminated DTO: enabled has exactly `enabled` and `provider_name`; disabled has exactly `enabled: false`. Gazel's payload and `/api/info` contract stay compatible. Flowl's payload/parser change atomically. The loopback policy determines cookie `Secure`; request/forwarded headers never do.

**Alternative:** normalizing the issuer changes identity; unifying by making Flowl accept production HTTP would weaken an explicit contract.

### 4. Guard every provider request and retain the cached-client model

Wrap the redirect-disabled Rustls client in an injectable `AsyncHttpClient` transport that validates the destination before sending. Use it consistently for discovery, initial JWKS, token exchange, and targeted JWKS refresh. Metadata validation alone is too late for the JWKS fetched inside `discover_async`. Validate the browser authorization endpoint separately before accepting startup metadata.

Reject credentials/fragments and apply the supplied transport policy. Require standalone `code`, authorization-code grant support when grants are advertised, usable advertised signing keys, and no advertised `none`. This signing-metadata policy is distinct from cryptographic verification: tests must exercise actual unsigned tokens rather than infer acceptance from metadata alone.

Keep the existing generation-aware cached `CoreClient`, single-flight refresh, one verification retry, retained usable keys, and monotonic failed-refresh cooldown. Preserve Basic/Post selection on rebuild. Keep the library's strict additional-audience rejection and add explicit raw-issuer, positive-`iat`, matching-`azp`, and applicable `at_hash` checks. Do not introduce arbitrary freshness, audience allowlists, or verifier leeway.

**Alternative:** custom token verification or full rediscovery per callback adds risk and discards existing interoperability/refresh guarantees.

### 5. Coordinate the full attempt lifecycle without provider I/O under a lock

Represent each browser-bound attempt with lifecycle ownership distinct from its one-time exchange permission. Parsing and bound-state checks precede consumption; consumption permits only one exchange. A completion guard remains until the result is finalized or invalidated, allowing replacement/logout to revoke an already exchanging attempt.

Coordinate binding/registry changes and session persistence, including rollback, against the same browser lifecycle. Check ownership again immediately before rotating and persisting authenticated state. Logout invalidates both the current local session and associated pending/in-flight ownership. An older failure cannot roll back a newer binding. Track rotation under the same ownership so a late cookie/response cannot restore authenticated access after successful invalidation.

Do not assume that a process-global mutex refreshes request-local session snapshots. Add deterministic tests with independently preloaded sessions and delayed saves/provider responses; use an authoritative ownership check and reload/persistence strategy compatible with `tower-sessions`. Provider requests and unrelated browsers must not be held behind a login/logout critical section. The JWKS refresh mutex remains separate.

Cap pending transactions at 1,024, prune before checking, and reject saturation before removing a prior attempt. Completion/coordination records must be retired with their attempt/session lifecycle rather than retained forever. On a failed authenticated save, invalidate provisional state before returning `provider_unavailable`; on failed logout persistence, remove the browser cookie but return HTTP 500 rather than falsely reporting success.

**Alternative:** copying Flowl's existing `login_lock` alone does not prove safety for preloaded snapshots, rotation, or a callback completing after logout.

### 6. Centralize parsing and response hygiene

Strictly form-decode callback parameters and detect duplicates by recognized parameter name. Model success and provider-error shapes separately; permit one each of `error_description` and `error_uri`, ignore their values, and never fetch an error URI. Malformed/ambiguous inputs do not consume a valid attempt. A well-formed bound provider error consumes it without exchange.

Use fixed safe error categories. HTTP 5xx, transport, malformed service responses, and unexpected responses map to provider unavailable; a parsed OAuth rejection from a non-5xx response maps to authentication failure. Invalid parsed tokens/claims remain authentication failure. Never format arbitrary provider or validation errors into logs.

Apply `no-store` outside the `/auth` routing/session layer so unknown paths, unsupported methods, and middleware-generated errors receive it, including disabled-mode responses. Add it to authentication-required API responses and document redirects without changing other cache policies or API fallback text. Preserve existing namespace routing outcomes; cache headers do not justify exposing or newly redirecting a route.

**Alternative:** adding headers only to successful handlers misses framework-generated errors.

### 7. Share pure frontend rules without sharing app navigation or cache policy

`auth-core.ts` owns config decoding, safe-local-target validation, and login URL construction. Gazel keeps its top-level hard-navigation adapter and existing public API helper exports; Flowl keeps its SvelteKit replace-navigation and cache-purge adapters. Both suppress recursive expiry navigation and deduplicate exact `401` plus `AUTHENTICATION_REQUIRED` handling.

Form-decode a query parameter once, then validate the resulting serialized target without changing it. Limit that serialization, not wire query size or validation-decoded size, to 2,048 UTF-8 bytes. Validate raw/decoded controls, malformed encoding, path delimiters, nested path escapes, dot segments, and namespace roots/descendants; `/apiary` is not `/api`. Ordinary escaped query/fragment data remains valid. Shared fixtures include C1 controls and percent signs in query values.

Keep the manual provider action, safe failure states, signed-out suppression, loading/error behavior, and no protected hydration on `/login`. Frontend path/query/hash preservation remains supported; backend redirects preserve only received path/query.

**Alternative:** sharing Svelte pages or navigation side effects would couple branding and Flowl's offline privacy behavior unnecessarily.

## Risks / Trade-offs

- [Mirrored source can drift] -> Exact-file parity plus common vectors are required paired-review gates; a reusable crate remains a future packaging option.
- [Concurrency tests can give false confidence] -> Force preloaded snapshots, provider barriers, response ordering, persistence failure, and logout-after-consumption; do not rely only on concurrent replay tests.
- [Stricter validation changes edge-case inputs] -> Document serialized length/encoding rules and signing metadata rejection; invalid return targets safely become `/`.
- [Provider response classification changes Gazel UI state] -> Test malformed/redirect/5xx responses separately from parsed OAuth rejection and invalid tokens.
- [Process-local sessions do not support arbitrary replicas] -> Preserve restart invalidation and document single-instance/sticky-routing limitations; no durable auth migration is introduced.

## Migration Plan

1. Add regression/conformance coverage, implement behavior alignment in small steps, then move code into the common layout and mirror it to Flowl.
2. Deploy each app as its own complete backend plus embedded frontend artifact; no coordinated runtime rollout is needed. Flowl's config schema and frontend decoder must ship together.
3. Verify public login/assets, protected APIs, return navigation, provider failures, logout/replay, and exact expiry through the configured proxy. Run peer parity and each repository's full verification gate before human review.
4. Roll back the complete artifact if necessary. No database migration exists; restart replaces process-local sessions, keys, and pending attempts. Do not disable authentication as a rollback shortcut.
5. Leave optional provider auto-redirect to a separately approved change after both implementations meet this contract.
