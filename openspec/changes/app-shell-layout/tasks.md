## 1. Dependencies and setup

- [x] 1.1 Install `lucide-svelte` in `ui/package.json`
- [x] 1.2 Create `ui/src/lib/styles/reset.css` with minimal CSS reset (box-sizing, margin/padding reset, body defaults using design tokens, anchor/image normalizations)

## 2. Theme store

- [x] 2.1 Create `ui/src/lib/stores/theme.ts` with `themePreference` state (`light | dark | system`), `effectiveTheme` derived value, `setTheme()` function, and `initTheme()` setup function
- [x] 2.2 `setTheme()` SHALL update `localStorage` key `gazel.theme`, set `data-theme` attribute on `<html>`, and update `themePreference` state
- [x] 2.3 `initTheme()` SHALL read from `localStorage`, set up `matchMedia('prefers-color-scheme: dark')` listener, and apply initial theme
- [x] 2.4 System preference changes SHALL update `effectiveTheme` and `data-theme` only when preference is `system`
- [x] 2.5 Write vitest tests for theme store: default to system, persist to localStorage, ignore OS changes when explicit preference set, update on OS change when system

## 3. Theme initialization script

- [x] 3.1 Add inline `<script>` to `ui/src/app.html` before `%sveltekit.head%` that reads `localStorage.getItem('gazel.theme')`, resolves effective theme (checking `matchMedia` for system), and sets `data-theme` on `<html>` synchronously — prevents flash of wrong theme

## 4. Root layout

- [x] 4.1 Import `tokens.css` and `reset.css` in `ui/src/routes/+layout.svelte`
- [x] 4.2 Define layout CSS custom properties in `<style>`: `--nav-bottom-height`, `--safe-area-bottom`, `--nav-bottom-total`, `--sidebar-width`, `--content-width-narrow`, `--content-width-default`, `--content-width-wide`
- [x] 4.3 Build the sidebar markup: logo, CTA button (`+` / "Fill-up"), Dashboard nav item (`LayoutDashboard` icon), spacer, Settings nav item (`Settings` icon) at bottom
- [x] 4.4 Build the bottom bar markup: Dashboard nav item, center raised CTA circle, Settings nav item
- [x] 4.5 Implement `isActive(href)` function using `page` from `$app/state` — Dashboard active for `/` and `/vehicles/*`, Settings active for `/settings`
- [x] 4.6 Style mobile breakpoint (≤768px): hide sidebar, show bottom bar, content padding 16px, bottom padding clears nav + safe area
- [x] 4.7 Style tablet breakpoint (769–1279px): show 64px icon sidebar, hide bottom bar, content margin-left 64px, padding 24px
- [x] 4.8 Style widescreen breakpoint (≥1280px): expand sidebar to 200px with labels, upgrade content width tokens, content margin-left 200px, padding 32px
- [x] 4.9 Style CTA: raised 52px circle in bottom bar (mobile), accent button in sidebar (tablet), accent button with label in sidebar (widescreen)
- [x] 4.10 Call `initTheme()` from the layout's `$effect` or `onMount`

## 5. Reusable components

- [x] 5.1 Create `ui/src/lib/components/PageContainer.svelte` — accepts `width` prop (`narrow | default | wide`), applies `max-width: var(--content-width-{width})` and `margin: 0 auto`
- [x] 5.2 Create `ui/src/lib/components/EmptyState.svelte` — accepts `icon` (Lucide component), `heading`, `description` props and optional `action` snippet. Centered layout with spacing between icon/heading/description/action.
- [x] 5.3 Write vitest tests for PageContainer: renders children, applies correct max-width class for each width value
- [x] 5.4 Write vitest tests for EmptyState: renders heading and description, renders action when provided, omits action area when not provided

## 6. Placeholder pages

- [x] 6.1 Update `ui/src/routes/+page.svelte` (Dashboard) to use `PageContainer` with `width="wide"` and render an `EmptyState` with a placeholder message
- [x] 6.2 Create `ui/src/routes/settings/+page.svelte` with `PageContainer` and a placeholder heading

## 7. Verification

- [x] 7.1 Run `npm run format:check --prefix ui` and fix any formatting issues
- [x] 7.2 Run `npm run lint --prefix ui` and fix any lint errors
- [x] 7.3 Run `npm run check --prefix ui` and fix any type errors
- [x] 7.4 Run `cargo test` and verify all tests pass (backend + UI)
- [ ] 7.5 Manually verify responsive layout at mobile (≤768px), tablet (769–1279px), and widescreen (≥1280px)
- [ ] 7.6 Verify theme toggle works: light → dark → system, no flash on reload
