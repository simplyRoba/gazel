## Context

The Settings route currently fetches `/api/info` for both About metadata and conditional rendering of a dedicated `AuthenticationSettings` component. The root protected layout owns the responsive sidebar and mobile bottom navigation but does not currently resolve app info. The existing logout control is a normal browser form posting to `/auth/logout`, which is required to preserve backend session invalidation and redirect semantics.

See `proposal.md` for motivation and the capability deltas for observable behavior.

## Goals / Non-Goals

**Goals:**

- Let the protected root layout conditionally render logout from the same optional `AppInfo.auth_enabled` signal already used by Settings.
- Keep logout available in persistent responsive navigation chrome while separating it visually and semantically from route links.
- Reuse the existing `settings.authentication.signOut` translation to avoid introducing a replacement label during a placement-only change.
- Remove Settings-only component code and stale heading/description translations.

**Non-Goals:**

- Changing `/api/info`, `/auth/logout`, session invalidation, or redirect behavior.
- Adding an app-info store, identity state, user/account data, profile controls, or an account menu.
- Refactoring unrelated navigation, Settings, authentication, or translation code.

## Decisions

### Resolve authentication enablement in the protected layout

The root layout will call the existing `fetchAppInfo()` client after entering the protected application branch and retain only whether `auth_enabled === true` for logout visibility. A failed request leaves logout hidden, matching the current Settings behavior and avoiding a broken chrome state. The Settings page will continue fetching app info independently for About metadata; introducing shared app-info state would broaden this placement fix.

Alternative: move all app-info loading into a new global store. Rejected because it changes application data flow and cache semantics for little benefit in this narrow change.

### Keep native form-based logout

The chrome action will remain a `<form method="POST" action="/auth/logout">` with a submit button. This preserves top-level browser navigation and allows the backend to remain the sole owner of session destruction and the `/login?logged_out=1` redirect.

Alternative: call logout through the frontend API client and navigate programmatically. Rejected because it duplicates backend navigation behavior and can subtly change cookie/redirect handling.

### Treat logout as a standalone chrome action

The control will use the logout icon and existing translated Sign out text, with neutral/secondary colors and spacing or a separator that distinguishes it from route links. On sidebar layouts it will sit below Settings at the bottom of the sidebar. The responsive mobile treatment will remain inside persistent navigation chrome while preserving the prominence and ordering of Dashboard, fill-up CTA, and Settings.

Alternative: add an account/profile menu. Rejected because Gazel has no local users or account model and a menu containing one action adds misleading structure.

### Delete the Settings-only presentation

The `AuthenticationSettings` component and its focused tests will be removed, along with its import/render site. Its enabled/disabled and form assertions will move to root-layout coverage. The Authentication heading and description translation keys may be removed as unused; the existing `settings.authentication.signOut` key is retained and reused by the layout.

## Risks / Trade-offs

- [App-info failure hides a useful action] → Preserve the existing fail-closed UI behavior and avoid exposing logout when enablement is unknown; authentication-expiry handling remains centralized in the API client.
- [Responsive chrome becomes crowded] → Keep normal route items and the fill-up CTA unchanged, and use a compact standalone treatment for Sign out rather than presenting it as another route.
- [Settings and layout both request app info on Settings] → Accept the small duplicate request to avoid introducing shared state in a narrowly scoped change.
- [Legacy translation namespace remains Settings-oriented] → Retain it intentionally for compatibility and minimal translation churn; only the visible placement changes.

## Migration Plan

1. Add focused layout tests for enabled/disabled visibility, placement semantics, and the native logout form.
2. Move the logout action and app-info enablement check into the protected root layout.
3. Remove the Settings authentication section/component and its obsolete tests and translations.
4. Run frontend formatting, linting, type checking, and tests, then run strict OpenSpec validation.

Rollback is a frontend-only revert restoring `AuthenticationSettings`; no backend, schema, API, or data migration is involved.
