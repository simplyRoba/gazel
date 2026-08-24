## 1. Callback contract tests

- [ ] 1.1 Update the successful OIDC callback integration coverage in `tests/auth.rs` to assert `200 OK`, no `Location` header, `Content-Type: text/html; charset=utf-8`, `Cache-Control: no-store`, `Referrer-Policy: no-referrer`, an authenticated session `Set-Cookie` with `SameSite=Lax`, and a completion document containing a same-origin script navigation plus safe non-script fallback; verify the test fails against the current immediate-redirect behavior
- [ ] 1.2 Add callback assertions for preserving a validated path/query/hash `return_to`, rejecting unsafe targets during `/auth/login` so the successful completion targets `/`, retaining exact failure redirects, and excluding authorization code, token, nonce, state, client secret, session identifier, and session content from completion HTML and navigation referrer metadata; verify with the focused auth test target

## 2. Safe callback completion implementation

- [ ] 2.1 Add a small Rust response builder in `src/auth/flow.rs` for successful callback completion that returns `200 OK` without `Location`, sets `Content-Type: text/html; charset=utf-8`, `Cache-Control: no-store`, and `Referrer-Policy: no-referrer`, and navigates with `location.replace` only after the existing session rotation and authenticated-session insertion succeed
- [ ] 2.2 Serialize the already validated `return_to` safely for JavaScript and independently escape it for HTML fallback attributes, including script-terminator and quote-shaped local targets; verify unsafe values cannot escape either output context
- [ ] 2.3 Replace only the final successful callback redirect with the completion response and confirm all state/nonce/PKCE/token/session validation and failure redirect branches remain unchanged; verify `SameSite=Lax` remains configured in `src/auth/session.rs`

## 3. Authentication documentation

- [ ] 3.1 Keep the `core-authentication` change spec and design aligned with the implemented response contract, including the rationale that the completion document terminates the cross-site authorization redirect chain before the protected SPA for WebKit/PWA compatibility; verify with OpenSpec validation

## 4. Verification

- [ ] 4.1 Run the focused Rust authentication tests and then the full Rust test suite with `cargo test`
- [ ] 4.2 Run `cargo fmt -- --check`, `cargo clippy -- -D warnings`, `npm run format:check --prefix ui`, `npm run lint --prefix ui`, and `npm run check --prefix ui`
- [ ] 4.3 Run `openspec validate "2026-08-24-fix-oidc-callback-webkit-completion" --type change --strict` and confirm all change artifacts are complete with `openspec status --change "2026-08-24-fix-oidc-callback-webkit-completion"`
