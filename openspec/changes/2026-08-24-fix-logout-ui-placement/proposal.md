## Why

Logout is a session-level application action, but Gazel currently hides it inside Settings under an Authentication section. Moving it into the persistent application chrome makes it consistently available without implying that Gazel has local accounts or identity management.

## What Changes

- Remove the Authentication section and logout control from the Settings page.
- Add a visually distinct, non-destructive Sign out action to the persistent navigation chrome, positioned below the normal sidebar navigation items when built-in OIDC authentication is enabled.
- Reuse the existing translated `Sign out` label and remove translation requirements that existed only for the deleted Settings section.
- Preserve the existing top-level `POST /auth/logout` form submission and backend post-logout redirect behavior.
- Keep the UI identity-neutral: do not add profile/account concepts or an account menu.
- Update focused layout and Settings UI tests for the new placement and enabled/disabled visibility.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `ui-app-layout`: Place the conditional Sign out action in the persistent navigation chrome and define its presentation and behavior.
- `ui-settings`: Remove the Settings-specific Authentication/logout requirement while retaining the app-info authentication signal contract.
- `ui-i18n`: Associate the existing translated Sign out label with application chrome and remove translation requirements used only by the deleted Settings section.

## Impact

- Frontend layout/navigation in `ui/src/routes/+layout.svelte` and its tests.
- Settings page/component code and tests, including removal of the Settings-only authentication section.
- Authentication translation resources and parity tests, while retaining the existing Sign out label key/value.
- OpenSpec deltas for `ui-app-layout`, `ui-settings`, and `ui-i18n`.
- No backend endpoint, session, redirect, API response, dependency, local-user, profile, or account-menu changes.
