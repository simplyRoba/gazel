## 1. i18n infrastructure

- [x] 1.1 Create `ui/src/lib/i18n/en.json` with all ~120 translation keys extracted from existing components, pages, and stores (flat dot-separated key structure)
- [x] 1.2 Create `ui/src/lib/i18n/de.json` with German translations for all keys
- [x] 1.3 Create `ui/src/lib/i18n/index.ts` with `t(key, params?)` lookup function, locale switching, English fallback for missing keys, and key-as-fallback for completely missing keys
- [x] 1.4 Create `ui/src/lib/i18n/errors.ts` with `resolveError(error, t)` function that maps `ApiError.code` to `t('error.<CODE>')` with fallback to `error.message`
- [x] 1.5 Wire `t()` locale reactivity to `settings.locale` from the settings store so locale changes trigger re-renders

## 2. Backend: expand valid locales

- [x] 2.1 Add `"de"` to `VALID_LOCALES` in `src/api/settings.rs`
- [x] 2.2 Add integration test: `PUT /api/settings` with `{ "locale": "de" }` returns 200

## 3. Locale-aware formatting

- [x] 3.1 Refactor `formatDistance(value, unit, locale?)` in `ui/src/lib/format.ts` to use `Intl.NumberFormat` with optional locale parameter (default `'en'`)
- [x] 3.2 Refactor `formatVolume(value, unit, locale?)` to use `Intl.NumberFormat`
- [x] 3.3 Refactor `formatEfficiency(value, distanceUnit, volumeUnit, locale?)` to use `Intl.NumberFormat`
- [x] 3.4 Refactor `formatCurrency(value, currency, locale?)` to use `Intl.NumberFormat`
- [x] 3.5 Update all call sites of format functions to pass `settings.locale` as the locale parameter
- [x] 3.6 Update `formatDate()` in dashboard page to use `settings.locale` instead of `undefined`
- [x] 3.7 Update `MonthlyCostChart.svelte` to use `settings.locale` instead of hardcoded `"en"`

## 4. String extraction: layout and navigation

- [x] 4.1 Replace hardcoded strings in `ui/src/routes/+layout.svelte` with `t()` calls (nav labels, CTA, vehicle picker dialog)
- [x] 4.2 Replace hardcoded strings in `ui/src/lib/components/ModalDialog.svelte` with `t()` calls

## 5. String extraction: dashboard page

- [x] 5.1 Replace hardcoded strings in `ui/src/routes/+page.svelte` with `t()` calls (empty states, summary labels, badges, fillup cards)

## 6. String extraction: settings page

- [x] 6.1 Replace hardcoded strings in `ui/src/routes/settings/+page.svelte` with `t()` calls (all section labels, buttons, modals, import/export messages, toasts)
- [x] 6.2 Update language selector in settings to show both English and Deutsch chips
- [x] 6.3 Replace hardcoded strings in vehicle sub-pages (`new/+page.svelte`, `[id]/edit/+page.svelte`) with `t()` calls

## 7. String extraction: forms and components

- [x] 7.1 Replace hardcoded strings in `FillupModal.svelte` with `t()` calls
- [x] 7.2 Replace hardcoded strings in `FillupForm.svelte` with `t()` calls (labels, placeholders, validation errors, buttons, missed fill-up prompt)
- [x] 7.3 Replace hardcoded strings in `VehicleForm.svelte` with `t()` calls (fuel type labels, form labels, placeholders, buttons)
- [x] 7.4 Replace hardcoded strings in `ToastHost.svelte` with `t()` calls
- [x] 7.5 Replace hardcoded strings in chart components (`ChartsPanel`, `ChartCard`, `EfficiencyChart`, `MonthlyCostChart`, `FuelPriceChart`) with `t()` calls

## 8. String extraction: stores

- [x] 8.1 Update `vehicles.svelte.ts` to use `resolveError()` for error messages instead of hardcoded fallback strings
- [x] 8.2 Update `fillups.svelte.ts` to use `resolveError()` for error messages instead of hardcoded fallback strings

## 9. Tests

- [x] 9.1 Add vitest translation completeness test: assert `en.json` and `de.json` have identical key sets
- [x] 9.2 Add vitest tests for `t()` function: simple lookup, parameterized strings, missing key fallback, locale switching
- [x] 9.3 Add vitest tests for `resolveError()`: known code, unknown code, fallback behavior
- [x] 9.4 Update existing `format.ts` tests for new `locale` parameter (verify English default, German decimal/grouping)
- [x] 9.5 Update existing store tests to account for `resolveError()` usage

## 10. Verification and cleanup

- [x] 10.1 Run `npm run check --prefix ui` and fix any type errors
- [x] 10.2 Run `npm run lint --prefix ui` and `npm run format:check --prefix ui`, fix any issues
- [x] 10.3 Run `cargo fmt -- --check` and `cargo clippy -- -D warnings`, fix any issues
- [x] 10.4 Run `cargo test` (full suite including UI tests) and ensure all pass
