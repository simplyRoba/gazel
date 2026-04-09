# gazel 1.0 Roadmap

Ordered feature chunks for the 1.0 release. Each chunk is sized for a single
OpenSpec change. Work top-to-bottom -- later chunks depend on earlier ones.

Reference repo: [simplyRoba/flowl](https://github.com/simplyRoba/flowl) --
same author, same stack (Axum + SvelteKit + SQLite). Reuse patterns where noted.

---

## 1. Backend foundation

> Stand up the HTTP server, database, and configuration so every later chunk
> has infrastructure to build on.

- [x] Add Axum, SQLx, rust-embed dependencies
      -- flowl `Cargo.toml`: deps, `[lints.clippy]` pedantic config, `[profile.release]` LTO/strip
- [x] Read config from env (`GAZEL_PORT`, `GAZEL_DB_PATH`, `GAZEL_LOG_LEVEL`)
      -- flowl `src/config.rs`: `ConfigSource` trait for testable config, `parse_or` helper
- [x] Connect to SQLite and run migrations on startup
      -- flowl `src/db.rs`: WAL journal mode, busy timeout, `create_if_missing`, auto-create parent dirs
- [x] `GET /health` endpoint (returns 200 + version)
      -- flowl `src/server.rs`: health check queries actual DB (`SELECT 1`)
- [x] Serve embedded SvelteKit SPA as fallback for non-API routes
      -- flowl `src/embedded.rs`: rust-embed with exact-path-then-index.html fallback
- [x] Router with access log middleware and graceful shutdown
      -- flowl `src/server.rs`: `access_log` middleware (method, path, status, latency), shutdown on SIGINT/SIGTERM
- [x] `AppState` with `FromRef` for pool extraction
      -- flowl `src/state.rs`: `FromRef<AppState> for SqlitePool` so handlers extract just the pool
- [x] `ApiError` type with JSON `{ "code": string, "message": string }` responses
      -- flowl `src/api/error.rs`: enum variants, `db_error()` helper, `JsonBody<T>` custom extractor
- [x] Integration test harness (`tests/common/mod.rs` with `test_app()`)
      -- flowl `tests/common/mod.rs`: in-memory pool, `json_request()`, `body_json()` helpers
- [x] UI test bridge (`tests/ui.rs`)
      -- flowl `tests/ui.rs`: runs `npm test`, auto-installs node_modules

## 2. Design system

> Establish the visual language before building any UI. Defines tokens and
> conventions all components will use. Gazelle-inspired: sleek, elegant,
> minimal.

- [x] Color palette -- warm neutrals (sand, stone, charcoal) with a refined
      accent (amber/gold or warm copper). Light and dark variants.
- [x] Typography -- select a clean sans-serif (e.g., Inter or DM Sans), define
      type scale (xs through 2xl), weights, line heights
- [x] Spacing scale -- consistent 4px-base scale (4, 8, 12, 16, 24, 32, 48, 64)
- [x] Border radii, shadows, transitions
- [x] CSS custom properties file (`ui/src/lib/styles/tokens.css`) with all tokens
- [x] Color mode -- CSS custom properties that swap via `[data-theme]` attribute,
      default to system preference (`prefers-color-scheme`)
- [x] Logo -- design a minimal, recognizable mark inspired by a gazelle silhouette.
      Produce SVG source, favicon (16/32/48), app icons (192/512 PNG), and
      OpenGraph image. Place in `docs/assets/` and wire into UI.
- [x] Document palette, type scale, and spacing in `CONTRIBUTING.md` design section

## 3. App shell and layout

> Responsive scaffold that all pages live inside. Navigation, page structure,
> and the first real UI components.

- [x] Root layout with `<header>`, `<main>`, `<nav>` structure
- [x] Responsive navigation -- bottom bar on mobile, sidebar or top bar on desktop
- [x] Page container component with consistent padding/max-width
- [x] Empty state component (reusable: icon, heading, description, action)
- [x] CSS Modules setup and conventions (one `.module.css` per component)
- [x] Favicon and app icon from existing logo asset

## 4. Vehicle API

> Backend CRUD for vehicles. The core entity everything else attaches to.

- [x] `vehicles` table migration (id, name, make, model, year, fuel_type,
      notes, created_at, updated_at)
      -- flowl `migrations/`: timestamp-prefixed, append-only pattern
- [x] Separate `VehicleRow` (DB) and `Vehicle` (API response) types with `From` impl
      -- flowl handler pattern: `PlantRow` -> `Plant` conversion for computed fields
- [x] Shared SQL fragment as `const`
      -- flowl `src/api/plants.rs`: `PLANT_SELECT` pattern
- [x] `GET /api/vehicles` -- list all
- [x] `GET /api/vehicles/:id` -- single vehicle
- [x] `POST /api/vehicles` -- create (validate required fields, return `201 Created`)
      -- flowl handler pattern: boundary validation functions, `(StatusCode::CREATED, Json<T>)`
- [x] `PUT /api/vehicles/:id` -- full update
- [x] `PATCH /api/vehicles/:id` -- partial update with `Option<Option<T>>` for nullable fields
      -- flowl `src/api/plants.rs`: `deserialize_nullable` for absent-vs-null-vs-value semantics
- [x] `DELETE /api/vehicles/:id` -- delete (cascade fill-ups or reject if has data?)
- [x] Integration tests for all endpoints
      -- flowl `tests/plants.rs`: oneshot pattern, assert status first then JSON fields

## 5. Vehicle management UI

> Frontend for adding, viewing, editing, and deleting vehicles.

- [x] API client functions in `ui/src/lib/api.ts`
      -- flowl `ui/src/lib/api.ts`: centralized `request()` helper, `ApiError` class with `status` + `code`
- [x] Vehicle store (`ui/src/lib/stores/vehicles.ts`) with Svelte 5 runes
      -- flowl `ui/src/lib/stores/plants.ts`: writable + async actions + error clearing + immutable updates
- [x] Vehicle list page (`/vehicles`) with cards showing name, make/model/year
- [x] Add vehicle form (modal or dedicated page)
- [x] Edit vehicle form
- [x] Delete vehicle with confirmation
- [x] Empty state when no vehicles exist ("Add your first vehicle")
- [x] Vitest tests for store and key components
      -- flowl `ui/src/lib/stores/plants.test.ts`: mock API module, assert store values with `get()`

## 6. Fill-up API

> Backend CRUD for fuel fill-ups, linked to a vehicle.

- [x] `fillups` table migration (id, vehicle_id FK, date, odometer,
      fuel_amount, fuel_unit, cost, currency, is_full_tank, is_missed,
      station, notes, created_at, updated_at)
- [x] `GET /api/vehicles/:id/fillups` -- list for vehicle (paginated, sorted by date desc)
- [x] `GET /api/vehicles/:id/fillups/:fillup_id` -- single fill-up
- [x] `POST /api/vehicles/:id/fillups` -- create (validate odometer > previous, etc.)
- [x] `PUT /api/vehicles/:id/fillups/:fillup_id` -- update
- [x] `DELETE /api/vehicles/:id/fillups/:fillup_id` -- delete
- [x] Integration tests

## 7. Fill-up logging UI

> Frontend for recording and browsing fill-ups per vehicle.

- [ ] API client functions for fill-ups
- [ ] Fill-up store per vehicle
- [ ] Vehicle detail page (`/vehicles/:id`) showing fill-up history
- [ ] Add fill-up form (date, odometer, amount, cost, station, full tank toggle)
- [ ] Edit/delete fill-up
- [ ] Fill-up list with key info at a glance (date, cost, amount, efficiency)
- [ ] Empty state for vehicle with no fill-ups
- [ ] Vitest tests

## 8. Settings and preferences

> User-selectable units, currency, and color mode. Persisted server-side so
> they survive across devices/browsers.

- [x] `settings` table migration (singleton row: unit_system, currency,
      distance_unit, volume_unit, color_mode, locale)
- [x] `GET /api/settings` -- read current settings
- [x] `PUT /api/settings` -- update settings
- [x] Settings page (`/settings`) with unit system (imperial/metric/custom),
      currency selector, color mode toggle (light/dark/system), language selector
- [x] Wire color mode toggle to `[data-theme]` on `<html>`
- [x] Persist color mode preference, apply on page load (no flash)
- [x] Frontend reads settings on app init, stores globally
- [x] Unit formatting utilities (`formatDistance()`, `formatVolume()`,
      `formatEfficiency()`, `formatCurrency()`)
- [x] Integration + vitest tests

## 9. Efficiency and cost calculations

> Backend calculation engine and API for fuel efficiency and cost metrics.
> Depends on fill-ups and settings (unit system).

- [ ] Efficiency calculation: distance / fuel between consecutive full-tank
      fill-ups (handle missed fill-ups gracefully)
- [ ] Cost-per-distance calculation
- [ ] Aggregation queries: average efficiency, total cost, total fuel,
      total distance -- per vehicle, over time ranges (month, year, all-time)
- [ ] `GET /api/vehicles/:id/stats` -- vehicle summary stats
- [ ] `GET /api/vehicles/:id/stats/history` -- time-series data for charting
- [ ] Stats respect the user's unit system
- [ ] Integration tests for calculation correctness (edge cases: single fill-up,
      missed fill-ups, partial tanks)

## 10. Dashboard

> The home page. Aggregated overview across all vehicles.

- [ ] Dashboard page (`/` or `/dashboard`)
- [ ] Summary cards: total vehicles, total spend (this month / all-time),
      best/worst efficiency across fleet
- [ ] Per-vehicle quick stats row (last fill-up date, current efficiency, monthly cost)
- [ ] Recent activity feed (last N fill-ups across all vehicles)
- [ ] Quick action: "Add fill-up" shortcut with vehicle selector
- [ ] Responsive grid layout -- cards reflow for mobile
- [ ] Vitest tests

## 11. Charts and visualization

> Trend charts for efficiency, cost, and fuel price over time per vehicle.

- [ ] Select and integrate a chart library (evaluate: Chart.js, LayerCake, uPlot)
- [ ] Efficiency trend chart (line chart, per vehicle, over time)
- [ ] Cost trend chart (bar or line, monthly spend per vehicle)
- [ ] Fuel price trend chart (cost per unit of fuel over time)
- [ ] Charts on vehicle detail page
- [ ] Summary chart(s) on dashboard (fleet-wide or selectable vehicle)
- [ ] Responsive chart sizing
- [ ] Vitest tests for data transformation logic

## 12. Data export

> Export vehicle and fill-up data as JSON.

- [ ] `GET /api/export` -- export all data as JSON (vehicles + fill-ups)
- [ ] `GET /api/vehicles/:id/export` -- export single vehicle + fill-ups
- [ ] Documented JSON schema for the export format
- [ ] Download button in UI (settings page or per-vehicle)
- [ ] Integration tests

## 13. Data import

> Import data from a JSON file matching the export schema.

- [ ] `POST /api/import` -- upload JSON, validate against schema
- [ ] Preview step: show what will be imported (N vehicles, M fill-ups)
      and any conflicts/duplicates
- [ ] Confirm step: actually write to database
- [ ] Import page or modal in settings
- [ ] Handle merge vs. replace semantics
- [ ] Integration tests with valid/invalid/partial payloads

## 14. Internationalization (i18n)

> Multi-language support. English as default, structure supports adding
> languages easily.

- [ ] Translation file structure and loading mechanism
      -- flowl `ui/src/lib/i18n/`: look at how translations are structured, loaded, and switched
- [ ] Translation keys for all existing UI strings (extract from components)
- [ ] Language selector in settings (wired to `locale` in settings table)
- [ ] `resolveError()` for i18n-aware API error messages
      -- flowl `ui/src/lib/stores/errors.ts`: maps `ApiError.code` to translated message, falls back to generic key
- [ ] Error codes in backend map to translation keys (not raw English strings)
      -- flowl `src/api/error.rs`: variants carry `&'static str` code, `default_message()` maps to English
- [ ] Date/number formatting respects locale
- [ ] Vitest tests for translation completeness (all keys present in all languages)

## 15. Polish and release prep

> Final pass before 1.0 tag.

- [ ] Audit all pages for responsiveness (phone, tablet, desktop)
- [ ] Keyboard navigation and basic a11y (labels, focus management, contrast)
- [ ] Loading states and error handling for all async operations
- [ ] Empty states for every list/page
- [ ] Favicon, OpenGraph meta, app manifest
- [ ] Update README with feature overview and screenshots
- [ ] Update CONTRIBUTING.md design section with final tokens
- [ ] Review and close any remaining OpenSpec changes
- [ ] Version bump to 1.0.0

---

## Design principles (for reference when creating changes)

| Principle | Meaning |
|---|---|
| **Sleek** | Like a gazelle -- graceful, not heavy. Minimal chrome, generous whitespace. |
| **Warm minimal** | Clean but not cold. Warm neutrals over blue-grays. |
| **Data-clear** | Numbers and stats are the hero. Typography does the heavy lifting. |
| **Responsive-equal** | Mobile and desktop are first-class citizens, designed together. |
| **Progressive** | Works with one vehicle and one fill-up. Scales gracefully to many. |

## Style direction (for the design system change)

- **Palette:** Warm neutrals (sand, stone, warm gray, charcoal) + accent (amber/gold
  or warm copper). Inspired by the tawny tones of a gazelle.
- **Typography:** Clean sans-serif. Crisp headings, readable body text. System font
  stack as fallback.
- **Spacing:** 4px base grid. Consistent rhythm.
- **Radii:** Subtle rounding (4-8px). Not pill-shaped, not sharp.
- **Shadows:** Minimal. Elevation only where it communicates hierarchy.
- **Motion:** Subtle, functional transitions. No decorative animation.
- **Color mode:** System preference by default, manual override. No flash on load.
- **CSS:** CSS Modules for component scoping, CSS custom properties for tokens.

## flowl reference index

Patterns from [simplyRoba/flowl](https://github.com/simplyRoba/flowl) to
reuse or adapt. Listed by flowl file path.

| flowl file | Pattern | Used in chunk |
|---|---|---|
| `Cargo.toml` | Clippy pedantic config, release profile (LTO, strip) | 1 |
| `build.rs` | `SKIP_UI_BUILD`, auto `npm install`, stub index.html | 1 |
| `src/config.rs` | `ConfigSource` trait, `parse_or` typed defaults, testable config | 1 |
| `src/db.rs` | SQLite pool (WAL, busy timeout, create-if-missing), migration runner | 1 |
| `src/embedded.rs` | rust-embed SPA handler (exact path -> index.html fallback) | 1 |
| `src/server.rs` | Router setup, `access_log` middleware, health check, graceful shutdown | 1 |
| `src/state.rs` | `AppState` + `FromRef<AppState> for SqlitePool` | 1 |
| `src/api/error.rs` | `ApiError` enum, `JsonBody<T>` extractor, `db_error()` helper | 1, 14 |
| `src/api/mod.rs` | RESTful route registration, per-route body limits | 4, 6 |
| `src/api/plants.rs` | `Row` vs response type, `From` impl, shared SQL `const`, `Option<Option<T>>` PATCH, boundary validation, `deserialize_nullable` | 4, 6 |
| `tests/common/mod.rs` | `test_app()`, `test_pool()`, `json_request()`, `body_json()` | 1 |
| `tests/ui.rs` | UI test bridge (runs npm test from cargo test) | 1 |
| `tests/plants.rs` | Integration test style (oneshot, status-first assertions) | 4, 6 |
| `ui/src/lib/api.ts` | `request()` helper, `ApiError` class (status + code), typed API functions | 5, 7 |
| `ui/src/lib/stores/plants.ts` | Writable stores, async actions, error clearing, immutable updates | 5, 7 |
| `ui/src/lib/stores/errors.ts` | `resolveError()` maps error codes to i18n messages | 14 |
| `ui/src/lib/stores/plants.test.ts` | Mock API module, assert store values with `get()` | 5, 7 |
| `ui/src/lib/i18n/` | Translation structure, loading, locale switching | 14 |
| `migrations/` | Timestamp-prefixed, append-only migration files | 4, 6, 8 |
| `Dockerfile` | Slim Debian, non-root, healthcheck, `TARGETARCH` multi-platform | 15 |
| `svelte.config.js` | Static adapter, fallback index.html | 1 |
| `vite.config.ts` | Dev proxy to Rust backend, jsdom test env | 1 |
