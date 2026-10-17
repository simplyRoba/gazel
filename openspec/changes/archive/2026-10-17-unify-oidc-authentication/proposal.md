# Proposal

## Why

Gazel and Flowl implement the same identity-neutral OIDC access gate, but independent implementations have accumulated different security guarantees and edge-case behavior. Consolidate them before adding optional automatic provider redirection so future authentication changes can be reviewed and tested against one common contract.

## What Changes

- Converge both repositories on synchronized authentication modules and common behavioral test vectors, retaining standalone builds rather than introducing a shared crate or sibling-path dependency.
- Combine Gazel's guarded outbound transport and explicit component boundaries with Flowl's bounded transactions, coordinated session persistence, explicit claim checks, and authentication-response cache protection.
- Bound pending login transactions to 1,024, preserve active transactions on saturation, and specify replacement, replay, persistence-failure, and logout/callback concurrency outcomes.
- Reject malformed, duplicate, or mixed callback parameters before token exchange; accept distinct standard provider-error detail fields without reflecting them.
- Standardize safe diagnostics and failure classification: parsed OAuth rejection is authentication failure; unusable provider service responses are provider unavailable.
- **BREAKING**: Tighten return-target acceptance to a 2,048-byte serialized target before validation decoding, reject encoded path confusion, and apply equivalent frontend/backend rules. Previously accepted longer encoded or ambiguous targets fall back to `/`.
- Make strict rejection of non-Unicode auth configuration explicit while leaving unrelated lenient configuration unchanged.
- Preserve Gazel's HTTP-loopback development support, cookie name, public/protected routes, `/auth/config` shape, `/api/info` compatibility, and explicit manual login/logout experience. Preserve Flowl's distinct production HTTPS and offline-cache policies in the coordinated peer change.
- Defer optional automatic provider redirection, distributed sessions, local users, provider logout, shared-crate packaging, and configurable network resource limits.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `core-authentication`: Common discovery, claims, bounded browser-bound transaction lifecycle, callback validation, safe return targets, session failure behavior, caching, diagnostics, and logout concurrency guarantees.
- `core-configuration`: Explicit rejection of present non-Unicode authentication values, especially the enable flag, without changing disabled-mode or ordinary configuration compatibility.
- `ui-login`: Validate return targets before constructing the provider action using the common navigation rules, retaining manual retry and signed-out behavior.
- `ui-api-client`: Validate expiry return targets and suppress recursive login navigation while preserving exact-401 recognition, top-level navigation, and deduplication.

## Impact

- Backend: `src/auth/**`, auth parsing in `src/config.rs`, integration in `src/server.rs`, and authentication response headers; existing API error codes and fallback text remain compatible.
- Frontend: auth helpers currently in `ui/src/lib/api.ts`, login provider-action construction, and associated API/login tests; branding and application shell remain app-owned.
- Tests: local provider fixtures, callback/session concurrency and failure-injection coverage, shared return/callback vectors, and cross-repository core synchronization checks.
- Documentation: authentication setup and changed validation behavior in operator/developer docs. No database migration, new service, new auth framework, or runtime cross-repository dependency.
- Coordinated peer: Flowl's `openspec/changes/unify-oidc-authentication/`; implementation is separately reviewable in each repository and must not merge or push without human approval.
