## Why

Safari/WebKit can fail to make Gazel's `SameSite=Lax` authenticated session cookie available to the SPA when a successful OIDC callback immediately continues the provider redirect chain with a `303`. This is reported most visibly when Gazel is launched from an installed Homepage PWA on iPhone or iPad, where the SPA can enter a `401`/login loop even though provider authentication succeeded.

## What Changes

- Change only the successful OIDC callback response: return a minimal `200 OK` completion document instead of a `303` directly to `return_to`.
- Have that document perform a fresh same-origin navigation to the already validated `return_to`, using `location.replace()` or equivalent behavior and safely escaped script and HTML fallback contexts.
- Return no success `Location` header and set `Cache-Control: no-store` and `Referrer-Policy: no-referrer`.
- Exclude authorization code, state, nonce, tokens, client secret, cookie/session contents, and all other authentication material from the completion document.
- Keep the authenticated session cookie at `SameSite=Lax`. Existing callback validation and failed-callback behavior remain governed by the canonical `Callback validation is one-time and complete` requirement; existing `return_to` validation and open-redirect protection remain governed by the canonical `Return navigation cannot become an open redirect` requirement.
- Add focused Rust response coverage plus manual acceptance verification of the installed iPhone/iPad Homepage PWA flow and normal Safari, desktop-browser, and subsequent logout/login flows.

## Capabilities

### New Capabilities

<!-- None. -->

### Modified Capabilities

- `core-authentication`: A successful OIDC callback now completes through a non-cacheable HTML navigation boundary before the protected SPA loads.

## Impact

- Affects the Rust OIDC callback success response and its tests.
- Adds no frontend route or dependency and changes no cookie setting, protected API response shape, provider configuration, callback failure path, or authentication validation.
