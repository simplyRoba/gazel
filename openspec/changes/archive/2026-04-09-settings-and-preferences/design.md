## Context

gazel has vehicle CRUD but no user preferences. The theme store (`ui/src/lib/stores/theme.svelte.ts`) persists to `localStorage` only; the inline script in `app.html` prevents flash by reading `localStorage` before first paint. There is no backend settings table, no settings API, and no unit formatting. The upcoming fill-up and stats features (chunks 6-9) need unit-aware formatting before they ship.

The reference project (flowl) uses a `user_settings` singleton table with `theme` and `locale` columns, a `GET/PUT /api/settings` pair, and a theme store that dual-writes to `localStorage` + the API. gazel will adopt the same pattern, extended with unit/currency fields.

## Goals / Non-Goals

**Goals:**

- Persist user preferences (units, currency, theme, locale) server-side so they survive across devices.
- Provide a settings page where users control all preference fields.
- Keep theme flash prevention working -- the inline `app.html` script must not regress.
- Add unit formatting utilities ready for fill-up and stats surfaces.

**Non-Goals:**

- Multi-user / per-user settings -- gazel is single-user; one singleton row is sufficient.
- i18n translations -- locale is stored but translation infrastructure is a separate chunk (14).
- Fill-up or stats UI -- this change provides formatting utilities but does not build the pages that use them.
- PWA offline settings caching -- not in scope until polish (chunk 15).

## Decisions

### 1. Singleton settings row with seed migration

**Decision:** Create a `settings` table with a single row (`id = 1`), seeded via the migration itself with sensible defaults.

**Rationale:** flowl uses this exact pattern. A singleton row avoids INSERT-on-first-use logic -- every GET/PUT simply targets `WHERE id = 1`. The migration inserts the default row so the app works immediately.

**Alternatives considered:**
- *Config file or env vars:* Not user-changeable at runtime. Ruled out.
- *Key-value table:* Flexible but loses type safety and requires multiple queries. Overhead not worth it for ~6 fields.

**Schema:**

```sql
CREATE TABLE settings (
    id              INTEGER PRIMARY KEY CHECK (id = 1),
    unit_system     TEXT NOT NULL DEFAULT 'metric',
    distance_unit   TEXT NOT NULL DEFAULT 'km',
    volume_unit     TEXT NOT NULL DEFAULT 'l',
    currency        TEXT NOT NULL DEFAULT 'USD',
    color_mode      TEXT NOT NULL DEFAULT 'system',
    locale          TEXT NOT NULL DEFAULT 'en'
);

INSERT INTO settings (id) VALUES (1);
```

The `CHECK (id = 1)` constraint enforces the singleton invariant at the DB level.

### 2. Partial update with COALESCE (PUT, not PATCH)

**Decision:** Use `PUT /api/settings` with partial-update semantics via `COALESCE`. All fields are optional in the request body; absent fields keep their current value.

**Rationale:** flowl uses this approach. It's simpler than PATCH with `Option<Option<T>>` because settings fields are never nullable -- they always have a value. `COALESCE(?, current)` is a clean one-query pattern. The endpoint is idempotent and safe to retry.

**Alternatives considered:**
- *Strict PUT (all fields required):* Forces the client to send the full settings object every time. Annoying for single-field changes like toggling theme.
- *PATCH with RFC 7396 merge-patch:* Overkill for a flat singleton with no nullable fields.

### 3. Theme dual-persistence: localStorage (fast) + API (durable)

**Decision:** `setTheme()` writes to `localStorage` synchronously (so the next page load has the value immediately for flash prevention) and fires an async `PUT /api/settings` call (so the server has it for cross-device sync). On init, the settings store fetches from the API and reconciles: if the server theme differs from `localStorage`, the server wins and `localStorage` is updated.

**Rationale:** This is the flowl pattern. The inline script in `app.html` reads `localStorage` before SvelteKit boots -- that can never wait for an API call. But the server is the source of truth for cross-device scenarios.

**Reconciliation flow on app init:**
1. Inline script runs, reads `localStorage`, sets `data-theme`. (No flash.)
2. SvelteKit boots, settings store calls `GET /api/settings`.
3. If `server.color_mode !== localStorage.gazel.theme`: update `localStorage` and re-apply theme via `setTheme()` (the DOM attribute updates, but since the page is already painted there's at most a single repaint -- not a flash of unstyled content).
4. If server has default (`system`) and localStorage has explicit value from before the API existed: the server value wins on fetch. On the very first use, this means the user's existing localStorage preference may be overwritten by `system`. To handle the upgrade path, the `initTheme` call can detect "first sync" (no server-stored value yet) and push the localStorage value to the server instead.

**Alternative considered:**
- *Server-only persistence:* Would require the SPA to wait for an API response before rendering, introducing flash or a loading screen. Rejected.

### 4. Unit system with presets and custom override

**Decision:** `unit_system` is one of `metric`, `imperial`, or `custom`. When `metric` or `imperial` is selected, `distance_unit` and `volume_unit` are automatically set to the conventional values. When `custom`, the user picks individual units.

| unit_system | distance_unit | volume_unit |
|---|---|---|
| `metric` | `km` | `l` |
| `imperial` | `mi` | `gal` |
| `custom` | user choice | user choice |

**Rationale:** Most users just want "metric" or "imperial". Power users who want km + gallons can use custom. The backend stores all three fields independently; the frontend auto-fills distance/volume when a preset is selected.

### 5. Formatting utilities as pure functions

**Decision:** Create a `ui/src/lib/format.ts` module with pure functions: `formatDistance(value, unit)`, `formatVolume(value, unit)`, `formatEfficiency(value, distanceUnit, volumeUnit)`, `formatCurrency(value, currency)`. Each takes explicit parameters rather than reading from a store, making them easily testable. A thin wrapper or derived store can provide "format with current settings" convenience.

**Rationale:** Pure functions are trivially testable with vitest. Keeping them separate from the store avoids circular dependencies and makes them reusable in contexts where the store isn't available (e.g., SSR, tests).

### 6. Settings store with Svelte 5 runes

**Decision:** Create `ui/src/lib/stores/settings.svelte.ts` using Svelte 5 runes (`$state`, `$derived`), matching the pattern established by `theme.svelte.ts` and `vehicles.svelte.ts`. The store exposes `initSettings()` (called once in root layout), `getSettings()`, and `updateSettings()`.

**Rationale:** Consistency with existing stores. Runes give fine-grained reactivity without boilerplate.

### 7. Settings page route

**Decision:** Add `/settings` as a top-level route (sibling to the existing vehicle management under `/settings/vehicles/...`). The settings page is a form with sections: Display (theme, locale), Units (unit system, distance, volume), Currency.

**Rationale:** `/settings` already appears in the navigation (the LSP errors in `+layout.svelte` reference it). This change fulfills that route.

## Risks / Trade-offs

**[Risk] First-sync upgrade path** -- Users who set a theme before the settings API exists will have a localStorage value but no server record. On first `GET /api/settings`, the server returns the default (`system`).
→ **Mitigation:** On init, if the fetched server theme is `system` and localStorage has an explicit `light` or `dark`, push the localStorage value to the server via `PUT /api/settings`. This is a one-time reconciliation.

**[Risk] Race between inline script and API fetch** -- The inline script sets theme from localStorage before SvelteKit boots. The API fetch happens after. If they disagree, there's a brief repaint.
→ **Mitigation:** The repaint is a single attribute swap on `<html>`, which triggers a CSS custom property change. It's visually instant (sub-frame). Acceptable trade-off vs. delaying render.

**[Risk] Custom unit combinations may produce confusing efficiency displays** (e.g., km/gal).
→ **Mitigation:** Display the full unit label (e.g., "km/gal") so the user sees exactly what they chose. No silent normalization.

**[Risk] Currency is stored as a string code (e.g., "USD") but we don't validate against an exhaustive list.**
→ **Mitigation:** Validate against a curated list of currency codes on the backend. Start with USD and EUR; expand later as needed.

## Open Questions

- Should the settings migration use a fixed timestamp prefix or the next available? (Recommendation: fixed, e.g., `20260409000000_settings.sql`, matching the project's existing convention.)
- Should the `/settings` page include a "Reset to defaults" action? (Recommendation: defer to polish phase.)
