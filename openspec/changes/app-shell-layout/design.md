## Context

gazel has a running Axum backend serving an embedded SvelteKit SPA, a design system with CSS tokens (`ui/src/lib/styles/tokens.css`), PWA icons/manifest, and a responsive mockup (`docs/mockup-app.html`). The SvelteKit app currently renders a blank passthrough layout. The sibling project flowl has a mature layout implementation using the same three-breakpoint responsive pattern (≤768px mobile, 769–1279px tablet, ≥1280px widescreen) that we adopt here.

The app has 2 navigation destinations (Dashboard, Settings) plus a primary CTA (Add fill-up) embedded directly in the navigation — center circle in the mobile bottom bar, accent button in the desktop sidebar.

## Goals / Non-Goals

**Goals:**

- A responsive shell that all future pages live inside — build it once, every route inherits it
- Navigation that surfaces the primary user action (add fill-up) in every viewport
- Theme preference that persists and applies without flash
- Reusable layout and empty-state components to keep future page code DRY
- Match flowl's proven layout architecture closely to avoid re-learning and benefit from a tested pattern

**Non-Goals:**

- No actual page content yet (Dashboard and Settings pages stay as placeholder/empty states)
- No pull-to-refresh (can be added later like flowl did)
- No offline detection or network status indicators
- No i18n wiring — labels are hardcoded English strings for now
- No CSS Modules for the layout itself — the root layout uses scoped `<style>` like flowl; CSS Modules are for feature components later
- No animated sidebar collapse/expand — width is purely CSS-driven via media queries

## Decisions

### Root layout follows flowl's monolithic pattern

The root layout (`+layout.svelte`) contains the sidebar, bottom bar, and content area in a single file with a scoped `<style>` block. Navigation items are inline `<a>` tags, not extracted into components.

**Rationale:** flowl proves this scales to 70+ releases without needing decomposition. With only 2 nav items + 1 CTA, a separate Nav component adds abstraction without benefit. The layout is the one place where the full structure is visible at a glance.

**Alternative considered:** Separate `Sidebar.svelte` / `BottomBar.svelte` components. Rejected — adds file-switching overhead for what is logically one cohesive structure.

### Three-breakpoint responsive layout (same as flowl)

| Viewport | Sidebar | Content margin | Content padding |
|---|---|---|---|
| ≤768px (mobile) | Hidden — bottom bar shows | 0 | 16px, bottom: nav height + safe area |
| 769–1279px (tablet) | 64px icon-only, fixed left | 64px left | 24px |
| ≥1280px (widescreen) | 200px with labels, fixed left | 200px left | 32px |

Breakpoint values are hardcoded in `@media` rules (not CSS properties) — same as flowl.

**Rationale:** These breakpoints align with common device sizes and are proven stable across flowl's lifecycle. CSS custom properties for breakpoints aren't supported in `@media` rules without `@custom-media` (no broad support yet).

### CTA placement: center of bottom bar (mobile) / top of sidebar (desktop)

Mobile: The "Add fill-up" button is a 52px accent-colored circle that sits in the center of the bottom tab bar, raised 14px above the bar edge. This makes the most frequent action (logging a fill-up) always one tap away.

Tablet/desktop: The CTA is an accent-colored button directly below the logo in the sidebar. On widescreen, it shows a label ("Fill-up"); on tablet, it's icon-only (＋).

**Rationale:** Filling up is the primary and most frequent user action. Embedding it in the nav ensures it's always visible without competing with page content. The raised-circle pattern is well-established (Instagram, TikTok, etc.).

**Alternative considered:** Floating Action Button (FAB) in the bottom-right corner. Rejected — the FAB can overlap content and doesn't integrate as cleanly with the nav. The mockup confirmed the center-CTA pattern looks better.

### Content width tokens

Three max-width presets, defined as CSS custom properties in the layout, upgraded at the 1280px breakpoint:

| Token | Default | ≥1280px | Use |
|---|---|---|---|
| `--content-width-narrow` | 640px | 720px | Forms (fill-up, vehicle edit) |
| `--content-width-default` | 800px | 960px | Detail pages, settings |
| `--content-width-wide` | 1200px | 1400px | Dashboard, data-heavy pages |

**Rationale:** Same approach as flowl. Prevents content from stretching to unreadable widths on large screens while giving different page types appropriate breathing room.

### Theme initialization: inline script in `app.html`

A `<script>` block added to `app.html` (before SvelteKit hydrates) reads `localStorage.getItem('gazel.theme')` and falls back to `matchMedia('(prefers-color-scheme: dark)')`. It sets `data-theme` on `<html>` synchronously.

This ensures no flash of wrong theme (FOUC) because the attribute is set before the first paint.

**Rationale:** SvelteKit's `+layout.svelte` runs after hydration — too late to prevent a flash. The inline script approach is the standard solution and is what flowl uses.

### Theme store (`$lib/stores/theme.ts`)

A Svelte 5 runes-based store managing:
- `themePreference`: `'light' | 'dark' | 'system'` — persisted to `localStorage` key `gazel.theme`
- `effectiveTheme`: derived from preference + system media query
- `setTheme(pref)`: updates preference, persists, and applies `data-theme` to `<html>`
- `initTheme()`: called on layout mount, sets up `matchMedia` listener for system changes

The store does NOT sync to the backend for now (no settings API yet). When settings are built, the store can be extended to persist server-side.

**Rationale:** localStorage-only is sufficient until the settings API exists. The store interface is designed so adding server sync later is additive, not a rewrite.

### Icons: lucide-svelte

Install `lucide-svelte` for tree-shakeable SVG icon components. Used icons for the shell: `LayoutDashboard`, `Settings`, `Plus`, `Package` (or similar for empty state).

**Rationale:** Same library as flowl. Tree-shakeable means only used icons end up in the bundle. Large icon set (1400+) covers future needs. Svelte-native components with `size` prop.

### Active nav state via `$app/state`

Use Svelte 5's `page` from `$app/state` (not the legacy `$page` store) to determine the active nav item:

```typescript
function isActive(href: string): boolean {
  if (href === '/') return page.url.pathname === '/' || page.url.pathname.startsWith('/vehicles');
  return page.url.pathname.startsWith(href);
}
```

Dashboard is active for `/` and `/vehicles/*` (vehicle detail is part of the dashboard flow). Settings is active for `/settings`.

**Rationale:** Matches flowl's pattern exactly. Using `$app/state` (runes) over `$page` (stores) aligns with Svelte 5 conventions.

### PageContainer component

A simple wrapper component that accepts a `width` prop (`'narrow' | 'default' | 'wide'`) and applies the corresponding `--content-width-*` token as `max-width`, with `margin: 0 auto` and consistent padding.

```svelte
<div class="page-container" style:max-width="var(--content-width-{width})">
  {@render children()}
</div>
```

**Rationale:** Avoids repeating max-width/padding/margin boilerplate in every page. Pages just wrap their content in `<PageContainer width="wide">`.

### EmptyState component

Accepts props: `icon` (Lucide component), `heading`, `description`, and an optional `action` snippet for a CTA button.

```svelte
<EmptyState icon={Car} heading="No vehicles yet" description="Add your first vehicle to start tracking.">
  {#snippet action()}<button>Add vehicle</button>{/snippet}
</EmptyState>
```

**Rationale:** Every list page needs an empty state. A consistent component prevents ad-hoc implementations and ensures visual consistency.

### Global CSS: minimal reset + base styles

A `reset.css` file imported in the layout:
- `*, *::before, *::after { box-sizing: border-box; margin: 0; padding: 0; }`
- `body` defaults: `font-family`, `font-size`, `line-height`, `color`, `background` from tokens
- `a { color: inherit; text-decoration: none; }`
- `img { display: block; max-width: 100%; }`

**Rationale:** Minimal reset — just enough to normalize browser defaults. Not a full CSS framework reset. Keeps the bundle small and predictable.

## Risks / Trade-offs

**Inline theme script adds non-module JS to `app.html`** → Acceptable trade-off for no-flash. The script is ~10 lines and runs synchronously before any rendering. This is the standard pattern for theme initialization in SPAs.

**Hardcoded breakpoint values can't be shared with JS** → Acceptable. We don't need JS-side breakpoint detection. If we ever do, `window.matchMedia` can use the same hardcoded values. CSS `@custom-media` may become viable in the future.

**Two nav items might feel sparse** → The CTA button fills the visual space and is the most important element. If more nav items are needed later (e.g., a separate vehicles page), adding them is trivial.

**No server-side theme persistence** → Users who clear localStorage lose their preference. This is temporary — the settings API (chunk 8) will add server persistence. The store is designed for easy extension.
