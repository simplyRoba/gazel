## Why

The backend foundation is running but the SvelteKit frontend is a blank page. Before any feature UI can be built, the app needs a responsive layout shell — navigation, page containers, theme wiring, and reusable base components. Every future page (dashboard, settings, fill-up forms) will live inside this shell. The design system tokens exist but aren't wired into the app yet.

## What Changes

- **Root layout** (`+layout.svelte`): Import design tokens, set up the `app-shell` structure with sidebar (tablet/desktop) and bottom bar (mobile), render content area with consistent padding and max-width constraints
- **Responsive navigation**: Three-state nav following flowl's proven pattern — bottom tab bar (≤768px), icon-only sidebar (769–1279px), expanded sidebar with labels (≥1280px). Two nav items: Dashboard and Settings. Center CTA button in bottom bar for "Add fill-up" on mobile, accent button in sidebar on desktop (as designed in the mockup)
- **Layout CSS custom properties**: Content width tokens (`--content-width-narrow`, `--content-width-default`, `--content-width-wide`), nav dimensions, safe-area handling for PWA
- **Theme initialization**: Inline script in `app.html` that reads stored preference or system `prefers-color-scheme` and sets `data-theme` on `<html>` before first paint (no flash). Theme store for runtime toggling.
- **Page container component**: Reusable wrapper with configurable max-width (narrow/default/wide) and consistent padding
- **Empty state component**: Reusable component with icon slot, heading, description, and action button — used on every list page when no data exists
- **Icon setup**: Install `lucide-svelte` for tree-shakeable SVG icons (same library as flowl)
- **Global CSS reset and base styles**: Minimal reset, body defaults using design tokens, global typography
- **Favicon wiring**: Already in `app.html` from previous change — verified working

## Capabilities

### New Capabilities

- `app-layout`: Root layout structure, responsive navigation (sidebar + bottom bar), content area, and layout CSS custom properties
- `theme-switching`: Theme preference detection, persistence (localStorage), runtime toggling, no-flash initialization, and `data-theme` attribute management
- `ui-components`: Reusable base UI components — PageContainer and EmptyState

### Modified Capabilities

_(none — no existing specs)_

## Impact

- **UI dependencies**: Add `lucide-svelte` to `ui/package.json`
- **UI files modified**: `ui/src/routes/+layout.svelte` (rewrite), `ui/src/app.html` (add theme init script)
- **UI files created**: `ui/src/lib/stores/theme.ts`, `ui/src/lib/components/PageContainer.svelte`, `ui/src/lib/components/EmptyState.svelte`, `ui/src/lib/styles/reset.css`
- **Tests**: Vitest tests for theme store logic. Component tests for EmptyState and PageContainer.
- **Backend**: No changes
- **Existing UI tests**: The existing dummy test in `ui/src/lib/dummy.test.ts` continues to pass
