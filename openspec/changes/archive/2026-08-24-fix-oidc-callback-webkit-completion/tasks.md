## 1. Callback response tests

- [x] 1.1 Update successful OIDC callback integration coverage in `tests/auth.rs` to assert `200 OK`, no `Location` header, `Content-Type: text/html; charset=utf-8`, `Cache-Control: no-store`, `Referrer-Policy: no-referrer`, an authenticated session `Set-Cookie` retaining `SameSite=Lax`, and a minimal completion document that performs same-origin `location.replace()` navigation with a safe HTML fallback; verify the test fails against the current immediate `303` response
- [x] 1.2 Add adversarial response coverage for safely preserving an already validated path/query/hash `return_to` in JavaScript and HTML contexts and for excluding authorization code, state, nonce, PKCE verifier, tokens, client secret, cookie/session identifiers, backend session contents, and other authentication material from the document and referrer metadata
- [x] 1.3 Confirm the existing callback-failure and `return_to` validation/open-redirect tests remain unchanged and passing

## 2. Safe callback completion implementation

- [x] 2.1 Add a small Rust success-response builder in `src/auth/flow.rs` that returns `200 OK` without `Location`, sets `Content-Type: text/html; charset=utf-8`, `Cache-Control: no-store`, and `Referrer-Policy: no-referrer`, and emits a minimal completion document using `location.replace()` or equivalent fresh navigation plus a safe non-script fallback
- [x] 2.2 Serialize only the already validated `return_to`, escaping it independently for JavaScript and HTML contexts without adding a dependency
- [x] 2.3 Replace only the final successful callback `303` with the completion response after existing validation, session rotation, and authenticated-session insertion; verify there is no frontend route, `SameSite=Lax` remains configured, and state/nonce/PKCE/session/token/issuer validation and every failed-callback branch are unchanged

## 3. Authentication documentation

- [x] 3.1 Keep the proposal, design, and `core-authentication` delta focused on the successful callback response boundary and its WebKit/PWA rationale; refer to canonical requirements rather than restating unchanged callback validation or open-redirect behavior
- [x] 3.2 Review `README.md` after implementation and update it only if the user-facing authentication guidance needs to describe this behavior

## 4. Verification and acceptance

- [x] 4.1 Run the focused Rust authentication tests and then the full Rust test suite with `cargo test`
- [x] 4.2 Run `cargo fmt -- --check`, `cargo clippy -- -D warnings`, `npm run format:check --prefix ui`, `npm run lint --prefix ui`, and `npm run check --prefix ui`
- [x] 4.3 Run `openspec validate "2026-08-24-fix-oidc-callback-webkit-completion" --type change --strict` and confirm all planning artifacts remain complete with `openspec status --change "2026-08-24-fix-oidc-callback-webkit-completion"`
