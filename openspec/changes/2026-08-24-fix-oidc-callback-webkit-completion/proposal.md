## Why

Safari/WebKit can fail to make a `SameSite=Lax` authenticated session cookie available after an OIDC redirect chain, particularly from installed iPhone/iPad PWAs and in-app browsers. Gazel currently establishes the session and immediately redirects from `/auth/callback`, leaving these clients loading indefinitely even though the provider authentication succeeded.

## What Changes

- Keep the authenticated session cookie at `SameSite=Lax` and preserve all existing CSRF, issuer, state, nonce, PKCE, token, and session validation.
- After a successful callback establishes the rotated session, return a minimal `200 OK` HTML completion document instead of an immediate `303` to `return_to`.
- Have the completion document perform a fresh same-origin navigation to the already centrally validated `return_to`, with safe fallback navigation markup.
- Safely serialize the validated destination, suppress referrer disclosure from the callback navigation, and ensure callback secrets, tokens, session data, and other sensitive values cannot appear in the completion HTML or navigation metadata.
- Preserve existing failed-callback redirects to `/login?error=...` and existing unsafe-target fallback to `/`.
- Add backend coverage and briefly document the WebKit/PWA redirect-chain rationale in the authentication specification.

## Capabilities

### New Capabilities

<!-- None. -->

### Modified Capabilities

- `core-authentication`: Successful OIDC callbacks complete with a same-origin HTML navigation boundary before entering the protected SPA, while failures and return-target validation retain their existing behavior.

## Impact

- Affects the Rust OIDC callback handler and its response tests.
- Updates the core authentication contract and design documentation; no frontend route or framework abstraction is required.
- No cookie attribute, API, provider configuration, or dependency changes are planned.
