## Context

The callback implementation in `src/auth/flow.rs` already validates and consumes the browser-bound transaction, exchanges and verifies the authorization code, cycles the session ID, inserts the authenticated session, and then calls the shared `303` redirect helper. Session cookies are configured with `SameSite::Lax` in `src/auth/session.rs`; that attribute and the existing validation boundary are security requirements, not implementation knobs.

See `proposal.md` for the WebKit/PWA motivation and `specs/core-authentication/spec.md` for the observable contract.

## Goals / Non-Goals

**Goals:**

- Add a narrow successful-callback response boundary that lets the browser commit the rotated authenticated cookie before navigating into the protected SPA.
- Preserve the existing callback failure redirects, centralized `return_to` validation, session lifecycle, and cookie attributes.
- Keep all completion markup independent of frontend routes and avoid reflecting or disclosing sensitive callback data, including through the navigation referrer.
- Cover successful, unsafe-target, failure, response-header, and sensitive-data cases with Rust tests.

**Non-Goals:**

- Do not change `SameSite`, add a second authentication cookie, or redesign CSRF protection.
- Do not change OIDC issuer, state, nonce, PKCE, token, JWKS, session expiry, or provider error handling.
- Do not add a SvelteKit route, client abstraction, or provider-specific workaround.

## Decisions

### Return a completion document only after successful session insertion

Keep every failure path in `callback` on `failure_redirect`. Replace only the final successful redirect after `cycle_id` and authenticated-session insertion with a response builder that returns `200 OK`, no `Location` header, `Content-Type: text/html; charset=utf-8`, `Cache-Control: no-store`, `Referrer-Policy: no-referrer`, and a small completion document. The document uses `location.replace(...)` so the callback URL is replaced rather than retained in history, allowing the browser to commit the session cookie before the protected navigation. The referrer policy prevents the callback URL, authorization code, and state from being sent as the `Referer` on that same-origin navigation.

**Alternative considered:** changing the cookie to `SameSite=None` would address some cross-site behavior but weakens the existing CSRF posture and is explicitly out of scope. A server-side delay or frontend route would be less deterministic and larger than the required boundary.

### Serialize the already validated destination in two output contexts

Use only `transaction.return_to`, which has already passed `validate_return_to`. Serialize it as a JSON string for the JavaScript argument, then escape HTML-sensitive characters (`<`, `>`, and `&`, as well as equivalent line-separator hazards) in the serialized representation so a path such as one containing a script terminator cannot escape the script element. Independently HTML-escape the destination when placing it in the `meta` refresh and link `href` fallback attributes. Do not include raw callback query values, transaction state, tokens, subject, or session data in the document.

**Alternative considered:** interpolating the destination directly into JavaScript or an HTML attribute is rejected because local-path validation does not by itself make either output context safe. A new templating or frontend dependency is unnecessary for this one response.

### Keep the fallback static, cache-resistant, and referrer-safe

Include a non-script fallback (a `meta` refresh and/or link) using the same escaped destination, plus a short “Continue” label. Set `Cache-Control: no-store` so an authentication transition document is not reused from a browser or intermediary cache, and set `Referrer-Policy: no-referrer` so script, meta-refresh, and link navigation cannot disclose callback parameters. The response contains no application state and is not a new frontend route.

### Test through the existing callback integration helpers

Update `tests/auth.rs` to assert the successful response status/body, session `Set-Cookie`, exact HTML content type, `Cache-Control: no-store`, `Referrer-Policy: no-referrer`, and absence of a success `Location` header, while retaining assertions that failed callbacks have the exact existing `Location`. Add coverage for a query/hash return target, an unsafe target rejected during `/auth/login` and stored as `/`, and absence of authorization code, token, nonce, state, client secret, and session-content markers in the body. Keep the existing cookie-attribute assertions, especially `SameSite=Lax`.

## Risks / Trade-offs

- [Risk] Older or restricted user agents may not execute the script or meta refresh. → Provide an escaped link fallback to the same validated local destination.
- [Risk] A future change to return-target validation could introduce characters unsafe for one output context. → Keep context-specific JSON/HTML escaping in the completion builder and test script-terminator/quote-shaped local targets.
- [Risk] Some clients may display a blank intermediate document. → Keep the body minimal, provide a visible fallback link, and use `location.replace` immediately.
- [Risk] Same-origin navigation normally carries the callback URL as a referrer. → Set and test `Referrer-Policy: no-referrer` on the completion response.
- [Risk] Tests that assume every successful callback has a `Location` header may fail. → Change only successful callback expectations; preserve all failure redirect assertions and explicitly assert `200 OK` with no `Location` for success.

## Migration Plan

No data migration or configuration change is required. Deploy the backend change normally; existing sessions and login transactions continue to use their current process-local storage and cookie attributes. If rollback is needed, reverting the callback response builder restores the previous immediate success redirect without changing stored data or provider configuration.
