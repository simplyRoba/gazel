## Context

Gazel currently returns a `303` from a successful `/auth/callback` directly to the validated `return_to`. In installed iPhone/iPad WebKit PWAs, continuing the provider redirect chain this way can cause the protected SPA request to arrive without the newly rotated `SameSite=Lax` authenticated session cookie, producing a `401`/login loop.

The canonical `Callback validation is one-time and complete` and `Return navigation cannot become an open redirect` requirements remain authoritative for callback validation, failure handling, and destination validation. This change modifies only the response emitted after callback processing and authenticated-session establishment have succeeded.

## Goals / Non-Goals

**Goals:**

- End the cross-site authorization redirect chain with a minimal successful HTML response before navigating into the protected SPA.
- Navigate freshly to the already validated same-origin destination without leaking callback or session material.
- Preserve the current `SameSite=Lax` session-cookie posture and every existing authentication validation and failure path.
- Verify the reported installed Homepage PWA behavior on actual iPhone/iPad WebKit in addition to Rust response tests.

**Non-Goals:**

- Do not change `SameSite` to `None`, add another authentication cookie, or weaken CSRF protection.
- Do not weaken or alter state, nonce, PKCE, session, token, issuer, audience, or signature validation.
- Do not change failed-callback behavior or `return_to` validation/open-redirect protection.
- Do not add a SvelteKit route, frontend abstraction, provider-specific branch, or dependency.

## Decisions

### Return a completion document only for callback success

After all existing callback validation, session rotation, and authenticated-session insertion have succeeded, return `200 OK` with `Content-Type: text/html; charset=utf-8`, `Cache-Control: no-store`, `Referrer-Policy: no-referrer`, and no `Location` header. The minimal document uses `location.replace()` (or equivalent fresh navigation) to enter the protected SPA, replacing the callback URL in browser history and allowing WebKit to begin a new same-origin navigation after committing the session cookie.

All failure branches continue to use the existing failure response. The authenticated cookie remains `SameSite=Lax`.

**Alternative considered:** `SameSite=None` is rejected because it weakens the existing cookie/CSRF posture and does not provide the requested navigation boundary. A delay or frontend route is less direct and increases surface area.

### Serialize only the already validated destination

The document receives only the `return_to` already accepted and stored by the existing centralized validation. Serialize the destination for JavaScript, then neutralize HTML-significant and script-termination characters in that serialized value. Independently HTML-escape the destination for any fallback attribute, such as a refresh target or link. A visible non-script continuation gives restricted user agents a safe fallback without introducing a frontend route or templating dependency.

Do not interpolate callback query values or emit authorization code, state, nonce, PKCE verifier, access token, ID token, client secret, cookie/session identifier, backend session content, or other authentication material. `Referrer-Policy: no-referrer` prevents the callback URL and query from becoming navigation metadata.

### Verify both the HTTP contract and the WebKit behavior

Rust integration tests will verify the successful status, headers, authenticated cookie, navigation target, context-safe escaping, and absence of authentication material. Existing tests for callback validation failures and unsafe `return_to` handling remain unchanged because those contracts are not modified.

Those tests cannot reproduce the installed WebKit/PWA cookie-commit bug. Acceptance therefore also requires a manual run against a TinyAuth-backed deployment on an actual iPhone or iPad:

`Installed Homepage PWA → open Gazel → Gazel login page → TinyAuth OIDC authorization → successful callback → completion document → Gazel SPA loads → protected API calls succeed → no 401/login loop`

The same deployment must also pass login in ordinary Safari, login in an ordinary desktop browser, and logout followed by login afterward.

## Risks / Trade-offs

- [Risk] Script execution may be disabled or restricted. → Include an independently escaped same-origin HTML fallback.
- [Risk] A validated local path may contain characters dangerous in script or HTML contexts. → Use context-specific escaping and adversarial response tests.
- [Risk] The callback URL could leak through navigation metadata. → Set and test `Referrer-Policy: no-referrer`; emit no authentication material in the body.
- [Risk] Automated response tests may pass while installed WebKit still loops. → Require the explicit physical-device PWA acceptance flow before considering the change complete.
- [Risk] The intermediate page may briefly appear. → Keep it minimal and navigate immediately with `location.replace()`.

## Migration Plan

No data or configuration migration is required. Deploy the backend response change normally. Reverting it restores the prior successful `303` response without changing stored sessions, provider configuration, or cookie attributes.
