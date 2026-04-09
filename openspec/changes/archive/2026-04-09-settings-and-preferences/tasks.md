## 1. Settings migration

- [x] 1.1 Create `migrations/20260409000000_settings.sql` with the `settings` table (`id`, `unit_system`, `distance_unit`, `volume_unit`, `currency`, `color_mode`, `locale`) including `CHECK (id = 1)` constraint and `INSERT INTO settings (id) VALUES (1)` seed row
- [x] 1.2 Verify migration runs successfully on a fresh in-memory database via existing test harness

## 2. Settings API endpoints

- [x] 2.1 Create `src/api/settings.rs` with `SettingsRow` (FromRow) and `Settings` (Serialize) response type, plus `From<SettingsRow> for Settings`
- [x] 2.2 Add `UpdateSettings` request struct with all fields as `Option<String>`, implement validation against allowed value domains (unit_system, distance_unit, volume_unit, currency, color_mode, locale)
- [x] 2.3 Implement `get_settings` handler: `GET /api/settings` querying `WHERE id = 1`, returning `Json<Settings>`
- [x] 2.4 Implement `update_settings` handler: `PUT /api/settings` with `COALESCE` partial-update pattern, validate before writing, return updated `Json<Settings>`
- [x] 2.5 Add error codes to `default_message()` in `src/api/error.rs`: `SETTINGS_INVALID_COLOR_MODE`, `SETTINGS_INVALID_UNIT_SYSTEM`, `SETTINGS_INVALID_DISTANCE_UNIT`, `SETTINGS_INVALID_VOLUME_UNIT`, `SETTINGS_INVALID_CURRENCY`, `SETTINGS_INVALID_LOCALE`
- [x] 2.6 Register routes in `src/api/mod.rs`: `/settings` with `get(get_settings).put(update_settings)`

## 3. Settings API integration tests

- [x] 3.1 Create `tests/settings.rs` with test for `GET /api/settings` returning defaults (200, all default values)
- [x] 3.2 Test `PUT /api/settings` updating a single field (color_mode), verify response reflects change and other fields unchanged
- [x] 3.3 Test `PUT /api/settings` updating multiple fields simultaneously
- [x] 3.4 Test `PUT /api/settings` with empty body `{}` returns 200 with unchanged values
- [x] 3.5 Test validation: invalid color_mode returns 422 with `SETTINGS_INVALID_COLOR_MODE`
- [x] 3.6 Test validation: invalid unit_system, distance_unit, volume_unit, currency, locale each return 422 with appropriate error code

## 4. Frontend API client functions

- [x] 4.1 Add `Settings` and `UpdateSettings` interfaces to `ui/src/lib/api.ts`
- [x] 4.2 Add `fetchSettings(): Promise<Settings>` function
- [x] 4.3 Add `updateSettings(data: UpdateSettings): Promise<Settings>` function (PUT)

## 5. Unit formatting utilities

- [x] 5.1 Create `ui/src/lib/format.ts` with `formatDistance(value, unit)`, `formatVolume(value, unit)`, `formatEfficiency(value, distanceUnit, volumeUnit)`, `formatCurrency(value, currency)` as pure functions
- [x] 5.2 Create `ui/src/lib/format.test.ts` with vitest tests covering all scenarios from the unit-formatting spec: km/mi, l/gal, km/L vs mpg vs mixed, USD/EUR/GBP, rounding, unknown currency fallback

## 6. Settings store

- [x] 6.1 Create `ui/src/lib/stores/settings.svelte.ts` with Svelte 5 runes: `$state` for settings fields, `initSettings()` that calls `fetchSettings()`, `getSettings()` getter, `updateSettings()` that optimistically updates local state and fires `PUT /api/settings`
- [x] 6.2 Handle `initSettings()` failure: fall back to client-side defaults (`metric`, `km`, `l`, `USD`, `system`, `en`)
- [x] 6.3 Create `ui/src/lib/stores/settings.test.ts` with vitest tests: init with mocked API, update propagation, fallback on API failure

## 7. Theme store server sync

- [x] 7.1 Modify `ui/src/lib/stores/theme.svelte.ts`: update `setTheme()` to dual-write -- `localStorage` synchronously + async `PUT /api/settings { color_mode }` (lazy-import api to avoid circular deps)
- [x] 7.2 Add reconciliation logic: accept a `serverColorMode` parameter in `initTheme()`, reconcile with localStorage value (server wins unless first-sync upgrade path applies)
- [x] 7.3 Wire reconciliation: settings store calls `initTheme(serverSettings.color_mode)` after fetching settings from API
- [x] 7.4 Update `ui/src/lib/stores/theme.test.ts` with tests for dual-write behavior and reconciliation scenarios (agree, disagree, first-sync upgrade, API unavailable)

## 8. Settings page UI

- [x] 8.1 Create `/settings` preferences section on the existing settings page with three groups: Display (theme, locale), Units (unit system, distance, volume), Currency
- [x] 8.2 Implement theme control using chip pattern (three sharp-cornered segments: Light, Dark, System) with active state highlighting, wired to `setTheme()`
- [x] 8.3 Implement unit system selector (Metric / Imperial / Custom chips) that auto-fills distance_unit and volume_unit for presets, enables individual selectors only for Custom
- [x] 8.4 Implement currency selector (dropdown or chips for the 10 supported currencies)
- [x] 8.5 Implement locale selector (currently only `en`, but wired for future expansion)
- [x] 8.6 Wire all controls to `updateSettings()` from the settings store; changes persist immediately on selection

## 9. App initialization

- [x] 9.1 Call `initSettings()` in the root layout `+layout.svelte` `onMount`, passing fetched `color_mode` to `initTheme()` for reconciliation
- [x] 9.2 Verify settings are available to child routes before they render preference-dependent content

## 10. Lint, test, and verify

- [x] 10.1 Run `cargo fmt -- --check` and fix any formatting issues
- [x] 10.2 Run `cargo clippy -- -D warnings` and fix any lint warnings
- [x] 10.3 Run `npm run format:check --prefix ui` and `npm run lint --prefix ui` and fix any issues
- [x] 10.4 Run `npm run check --prefix ui` and fix any type errors
- [x] 10.5 Run `cargo test` (full suite including UI tests) and ensure all tests pass
