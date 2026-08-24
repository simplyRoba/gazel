## 1. Focused UI Tests

- [ ] 1.1 Add root-layout tests for omitted and enabled `auth_enabled`, verifying Sign out is hidden in disabled mode and rendered in persistent chrome in enabled mode.
- [ ] 1.2 Add layout assertions that Sign out uses a top-level `POST /auth/logout` form, remains separate from route links, and has no account/profile menu or destructive treatment; run the focused layout test file to verify expected failures before implementation.

## 2. Relocate Logout to Application Chrome

- [ ] 2.1 Resolve the existing optional app-info authentication signal in the protected root layout, fail closed when app info cannot be loaded, and verify the focused enabled/disabled layout tests pass.
- [ ] 2.2 Render the existing translated Sign out action in persistent responsive navigation chrome, placing it below and visually separating it from normal sidebar navigation with neutral styling; verify focused layout tests and responsive DOM assertions pass.
- [ ] 2.3 Preserve the native `POST /auth/logout` form submission without frontend redirect handling and verify the form contract test passes.

## 3. Remove Settings Authentication Presentation

- [ ] 3.1 Remove the Authentication section from the Settings page and delete its Settings-only component and tests; verify no Authentication heading or logout action remains in Settings coverage.
- [ ] 3.2 Remove the unused Authentication heading/description translation keys while retaining the existing translated `settings.authentication.signOut` label in every locale, update translation-key tests, and run the i18n tests.
- [ ] 3.3 Review `README.md` for user-facing impact and confirm no documentation change is needed for this placement-only update.

## 4. Validation

- [ ] 4.1 Run `npm run format:check --prefix ui`, `npm run lint --prefix ui`, `npm run check --prefix ui`, and `npm run test --prefix ui`; fix any frontend failures.
- [ ] 4.2 Run `cargo fmt -- --check`, `cargo clippy -- -D warnings`, and `cargo test`; fix any repository-level regressions.
- [ ] 4.3 Run strict OpenSpec validation for `2026-08-24-fix-logout-ui-placement` and resolve all reported artifact or delta errors.
