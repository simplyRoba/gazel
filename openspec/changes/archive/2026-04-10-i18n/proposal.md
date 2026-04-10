## Why

All UI text is hardcoded English (~120 strings across pages, components, and stores). The locale setting is already plumbed through the full stack (DB → API → store → settings UI) but does nothing beyond validation. Adding i18n now -- before the 1.0 release -- avoids a larger retroactive extraction later and unlocks the German locale that the project needs.

## What Changes

- Introduce a translation system in the frontend (`ui/src/lib/i18n/`) with JSON translation files, a reactive locale store, and a `t()` lookup function.
- Extract all hardcoded English strings from Svelte components, pages, and stores into translation keys. English becomes the default/fallback translation file.
- Add German (`de`) as the first additional language to validate the system end-to-end.
- Build a `resolveError()` function that maps `ApiError.code` values to locale-aware messages, replacing direct use of backend `message` strings in the UI.
- Refactor `format.ts` to use `Intl.NumberFormat` and `Intl.DateTimeFormat` so number/date output respects the active locale (e.g., comma decimals for `de`).
- Expand `VALID_LOCALES` in the backend settings validation to accept `de` alongside `en`.
- Wire the settings language selector to switch the active locale at runtime and persist the choice.
- Add vitest coverage for translation completeness (all keys present in every language) and locale-aware formatting.

## Capabilities

### New Capabilities

- `i18n`: Translation infrastructure -- file structure, loading, `t()` function, locale switching, `resolveError()`, and translation completeness testing.

### Modified Capabilities

- `api/settings`: Expand `VALID_LOCALES` domain from `["en"]` to `["en", "de"]`.
- `ui/settings`: Wire language selector to switch active translation locale and show all supported languages.
- `ui/unit-formatting`: Formatting functions gain a `locale` parameter and use `Intl.NumberFormat` / `Intl.DateTimeFormat` for locale-aware output.
- `api/api-error-handling`: No backend code changes required -- the existing `code` + `default_message()` design already supports frontend-driven i18n. Delta spec documents the contract that error codes are the stable i18n key, and `message` is an English fallback only.

## Impact

- **Frontend**: Every component/page with user-visible text gains `t()` calls. `format.ts` API changes ripple to all call sites. New `ui/src/lib/i18n/` directory with `en.json`, `de.json`, and loader module.
- **Backend**: Single-line change to `VALID_LOCALES` in `src/api/settings.rs`. No schema migration needed (`locale` column already exists).
- **Tests**: New vitest suite for translation key parity. Existing format tests updated for locale parameter. Store tests updated to mock `resolveError()`.
- **Dependencies**: None -- `Intl` APIs are built-in; translation files are static JSON.
