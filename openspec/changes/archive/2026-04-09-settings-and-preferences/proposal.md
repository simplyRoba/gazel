## Why

gazel currently has no way for users to configure units, currency, or color mode -- and no backend persistence for preferences. Theme choice lives only in `localStorage`, so it doesn't survive across devices. Before fill-up logging lands (chunk 7-9), the app needs a settings surface and unit formatting so that distances, volumes, and costs display in the user's preferred system.

## What Changes

- Add a `settings` singleton table with columns for unit system, currency, distance unit, volume unit, color mode, and locale.
- Add `GET /api/settings` and `PUT /api/settings` endpoints (partial update semantics).
- Add a `/settings` page with controls for unit system (imperial / metric / custom), currency, color mode (light / dark / system), and locale.
- Wire the color mode toggle to `[data-theme]` on `<html>`, dual-persisting to both `localStorage` (for flash-free reload) and the server (for cross-device sync).
- On app init, fetch settings from the API and hydrate a global settings store; reconcile server-side theme with the `localStorage` value already applied by the inline script.
- Add unit formatting utilities (`formatDistance()`, `formatVolume()`, `formatEfficiency()`, `formatCurrency()`) driven by the settings store.

## Capabilities

### New Capabilities

- `api/settings` (`openspec/specs/api/settings/`): Backend settings CRUD -- migration, singleton row, GET/PUT endpoints, validation. Covers unit system, currency, distance/volume units, color mode, and locale.
- `ui/settings` (`openspec/specs/ui/settings/`): Frontend settings page, global settings store, app-init hydration, and the UI controls for all preference fields.
- `ui/unit-formatting` (`openspec/specs/ui/unit-formatting/`): Pure formatting utilities for distance, volume, efficiency, and currency that read from the settings store and are usable across all future UI surfaces.

### Modified Capabilities

- `ui/theme-switching` (`openspec/specs/ui/theme-switching/`): Theme preference gains server-side persistence via the settings API. `setTheme()` will write to both `localStorage` and `PUT /api/settings`. On init, the app reconciles the server-stored theme with the `localStorage` value to keep them in sync without introducing a flash.

## Impact

- **Backend**: New migration (`settings` table), new `src/api/settings.rs` module, route registration in `src/api/mod.rs`.
- **Frontend**: New settings store (`ui/src/lib/stores/settings.svelte.ts`), new settings page + route, new formatting utilities module, changes to the theme store to add server sync.
- **API contract**: Two new endpoints (`GET /api/settings`, `PUT /api/settings`) with a JSON shape covering all preference fields.
- **App init**: Root layout or `+layout.ts` will fetch settings on first load and seed the global store.
- **Tests**: Integration tests for the settings endpoints; vitest tests for the settings store, formatting utilities, and theme-server sync.
