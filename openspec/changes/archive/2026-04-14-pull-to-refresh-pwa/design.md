## Context

gazel ships as a standalone PWA (manifest.json `display: "standalone"`). On mobile devices, installed PWAs lose the browser's built-in pull-to-refresh gesture. Users currently have no intuitive way to refresh dashboard data without navigating away and back. The flowl project (same author, same stack) already solved this with a well-tested implementation that we replicate here.

The existing root layout (`+layout.svelte`) renders a sidebar/bottom-bar navigation shell with a `<main class="content">` area. No service worker exists; the PWA is a simple standalone wrapper around the SPA.

## Goals / Non-Goals

**Goals:**
- Replicate flowl's pull-to-refresh gesture exactly: same thresholds, elastic feel, state machine, and visual behavior.
- Keep all math/logic in a pure, side-effect-free module (`pull-to-refresh.ts`) for full testability.
- Gate activation behind standalone PWA mode + touch capability to avoid conflicts with browser pull-to-refresh or desktop usage.
- Integrate into the existing root layout without extracting a separate component (matches flowl's pattern).

**Non-Goals:**
- Custom refresh behavior per route (always does `window.location.reload()`).
- Service worker integration or offline support.
- Android/iOS install prompt.
- Pull-to-refresh on desktop or in-browser (non-standalone) contexts.
- Animated transitions beyond the CSS-only spinner and snap-back.

## Decisions

### 1. Pure logic module + layout integration (no component extraction)

**Decision**: All calculations, eligibility checks, and state derivation live in `ui/src/lib/pull-to-refresh.ts`. Touch event wiring, indicator markup, and CSS live inline in `+layout.svelte`.

**Rationale**: This matches flowl exactly. The indicator is tightly coupled to the layout's touch handlers and content offset. Extracting a component would create prop/event ceremony without benefit since only the root layout needs this feature.

**Alternatives**: A standalone `<PullToRefresh>` component was considered but rejected because the touch handlers need to manage the layout's `<main>` margin-top and the indicator needs fixed positioning relative to the viewport, both of which are layout concerns.

### 2. Exponential decay for elastic feel

**Decision**: Use `range * (1 - e^(-x/range))` for both indicator and content offsets past the 128px threshold.

- Indicator elastic range: 12px (MAX_OFFSET 140 - THRESHOLD 128). Feels "stuck" once armed.
- Content elastic range: 100px. Continues moving slowly past threshold for a natural feel.

**Rationale**: Direct port from flowl. The asymptotic approach gives the "rubber band" sensation expected on mobile. Two separate curves allow the indicator to cap tightly while content continues subtle movement.

### 3. Non-passive touchmove for scroll prevention

**Decision**: Register `touchmove` with `{ passive: false }` to call `event.preventDefault()` during an active pull gesture. Register `touchstart` as `{ passive: true }`.

**Rationale**: `preventDefault()` on touchmove is the only way to block the browser's native scroll/bounce during the pull gesture. touchstart doesn't need it, so it stays passive for performance. This is a standard pattern for custom gesture handling.

### 4. Eligibility: overlay detection via DOM query

**Decision**: `hasBlockingOverlay()` checks `document.querySelector('dialog[open]')` to detect any open modal dialog. gazel uses `<dialog>` elements with `.showModal()` for all overlays (ModalDialog, FillupModal, vehicle picker).

**Rationale**: DOM query is simple and catches any dialog without coupling to component state. The check runs on touchstart and each touchmove, which is acceptable at gesture frequency. gazel's overlays all use native `<dialog>` so this single selector covers all cases (unlike flowl which also checks custom class selectors for non-dialog overlays).

### 5. Route allowlist for gazel

**Decision**: Allow pull-to-refresh on `/` and `/settings` only. Block on `/settings/vehicles/new` and `/settings/vehicles/*/edit` (form pages).

**Rationale**: gazel has only 4 routes. The dashboard and settings are read-oriented pages where refresh is useful. Vehicle form pages (new/edit) should not trigger refresh to avoid data loss during form input. This matches flowl's pattern of blocking on edit/new routes.

### 6. Reload via `window.location.reload()` with 120ms delay

**Decision**: The refresh action is a full page reload after a 120ms delay.

**Rationale**: Direct port from flowl. The delay lets the "Refreshing..." spinner render visually before the reload blanks the page. A full reload is the simplest approach since gazel re-initializes settings and vehicles on mount anyway.

## Risks / Trade-offs

- **[Risk] Non-passive touchmove may affect scroll performance** -> Mitigated: only registered when gesture is active (standalone PWA + touch capable), and `preventDefault` is only called when pull distance > 0. Normal scrolling paths are unaffected.
- **[Risk] DOM query on every touchmove for overlay detection** -> Mitigated: `document.querySelector` is fast for a single selector, and touchmove fires at ~60Hz which is well within budget. Could be optimized to a reactive flag if needed, but flowl proves this approach works.
- **[Risk] `display-mode: standalone` media query not supported in all browsers** -> Mitigated: fallback to `navigator.standalone` (iOS Safari). Feature is additive; non-detection just means no pull-to-refresh (graceful degradation).
- **[Trade-off] Full page reload vs. targeted data refresh** -> Full reload is simpler and matches flowl. Targeted refresh would be more efficient but requires orchestrating multiple store reloads and doesn't provide visual confirmation of fresh data.
