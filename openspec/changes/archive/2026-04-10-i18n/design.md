## Context

All ~120 user-visible strings in the gazel UI are hardcoded English. The `locale` field is already stored in the `settings` table (default `"en"`), exposed via `GET/PUT /api/settings`, and rendered in the settings page -- but it does nothing beyond validation against `VALID_LOCALES = &["en"]`. The backend error system already uses structured `code` fields designed for frontend i18n mapping, with `default_message()` providing English fallbacks.

The flowl reference project uses a `ui/src/lib/i18n/` directory with static translation files, a `resolveError()` function that maps error codes to translation keys, and locale-driven formatting. This change follows the same pattern.

## Goals / Non-Goals

**Goals:**

- Introduce a translation system that works with static JSON files, zero runtime dependencies, and no build-time code generation.
- Extract all hardcoded English strings into translation keys with English as the default/fallback language.
- Add German (`de`) as the first additional locale to validate the system end-to-end.
- Build `resolveError()` to replace direct use of backend error `message` strings.
- Make number and date formatting locale-aware via `Intl` APIs.
- Keep the translation system simple enough that adding a new language requires only a new JSON file and one entry in `VALID_LOCALES`.

**Non-Goals:**

- No right-to-left (RTL) layout support.
- No pluralization engine or ICU MessageFormat -- simple string interpolation only (e.g., `{count}` placeholders).
- No lazy loading of translation files -- all translations are bundled.
- No server-side rendering of translated content -- translations are frontend-only.
- No automated translation tooling or CI extraction.
- No backend message translation -- `default_message()` remains English; the frontend overrides via `resolveError()`.

## Decisions

### 1. Static JSON translation files over a library

**Decision**: Use plain JSON files (`en.json`, `de.json`) in `ui/src/lib/i18n/` with a thin `t(key, params?)` lookup function.

**Alternatives considered**:
- **i18next / svelte-i18n**: Full-featured but adds ~15-30 KB to the bundle and introduces runtime complexity (namespaces, plugins, async loading). Overkill for a self-hosted single-user app with 2 languages.
- **Paraglide / Inlang**: Compile-time approach with great tree-shaking but requires build tooling integration and a different mental model.

**Rationale**: The app has ~120 strings and 2 locales. A hand-rolled `t()` function is <100 lines, zero-dependency, and fully type-safe with TypeScript. If the app grows past 5+ languages, migrating to a library is straightforward since the JSON key structure is compatible.

### 2. Flat key namespace with dot-separated hierarchy

**Decision**: Translation keys use a flat dot-separated structure matching the component/feature they appear in.

```json
{
  "nav.dashboard": "Dashboard",
  "nav.settings": "Settings",
  "dashboard.empty.title": "No vehicles yet",
  "dashboard.empty.description": "Add your first vehicle to start tracking fill-ups.",
  "fillup.form.date": "Date *",
  "error.VEHICLE_NOT_FOUND": "Vehicle not found."
}
```

**Rationale**: Dot-separated keys are easy to grep, sort, and compare across files. A flat structure (all keys at top level in JSON) avoids nested object traversal complexity in the `t()` function. Error codes map 1:1 to `error.<CODE>` keys.

### 3. Reactive locale via settings store

**Decision**: The active locale is driven by `settings.locale` from the existing settings store. When the user changes their language in settings, the store updates, `PUT /api/settings` persists the choice, and a reactive `$derived` re-renders all `t()` calls.

**Alternative**: Browser `navigator.language` detection with local override. Rejected because the app already has a server-persisted `locale` field, and using it ensures consistency across devices.

**Rationale**: The infrastructure already exists. The `t()` function reads from a reactive locale signal, so all UI updates automatically when the locale changes.

### 4. `resolveError()` maps error codes to translation keys

**Decision**: A `resolveError(error: ApiError, t: TranslateFunction)` function in `ui/src/lib/i18n/errors.ts` maps `error.code` to `t('error.<code>')`. If the key is missing, it falls back to `error.message` (the backend's English default).

**Rationale**: This keeps the backend unchanged. The `code` field was designed for this purpose from the start. Stores call `resolveError()` instead of using `e.message` directly.

### 5. Locale-aware formatting via `Intl` APIs

**Decision**: Refactor `format.ts` functions to accept an optional `locale` parameter (defaults to `'en'`). Use `Intl.NumberFormat` for number formatting and pass locale to `toLocaleDateString()` calls.

**Alternative**: Keep `toFixed()` and add a separate locale formatting layer. Rejected because `Intl.NumberFormat` handles decimal separators, grouping, and edge cases correctly across locales.

**Rationale**: `Intl` APIs are built-in, well-supported, and handle locale nuances (comma vs dot decimals, thousand separators) without any dependency. The function signatures gain one parameter but remain pure.

### 6. Translation completeness test

**Decision**: A vitest test loads all translation JSON files, extracts their key sets, and asserts that every key in `en.json` exists in every other locale file.

**Rationale**: This catches missing translations at test time rather than at runtime. The test is simple (set difference comparison) and runs in <100ms.

## Risks / Trade-offs

**[Risk] Large string extraction diff** → The PR that extracts ~120 strings into `t()` calls will touch nearly every component. Mitigated by doing extraction as a single focused task after the i18n infrastructure is in place, and keeping key naming consistent so review is mechanical.

**[Risk] Formatting function signature change** → Adding a `locale` parameter to `formatDistance()`, `formatVolume()`, `formatEfficiency()`, and `formatCurrency()` requires updating every call site. Mitigated by making `locale` optional with `'en'` default so existing tests pass without changes during the transition.

**[Risk] Incomplete German translations at launch** → Some strings may be missed or poorly translated. Mitigated by the completeness test (catches missing keys) and the fallback to English for any untranslated key.

**[Trade-off] No pluralization support** → Strings like `"{count} fill-up(s)"` use a simple `{count}` placeholder without plural forms. Acceptable for 2 languages where plural rules are similar. If languages with complex plural rules (e.g., Arabic, Polish) are added later, a pluralization function can be introduced without changing the JSON structure.

**[Trade-off] Bundle includes all locales** → Both translation files are always loaded. With 2 languages and ~120 keys each, the overhead is <5 KB total. Lazy loading would add complexity for negligible gain.
