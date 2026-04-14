## Why

gazel is served as a standalone PWA (`display: "standalone"` in manifest.json). On mobile touch devices, installed PWAs lack the browser's built-in pull-to-refresh gesture, so there is no way for users to refresh dashboard data without a manual page reload. Adding a native-feeling pull-to-refresh gesture -- identical to the implementation in flowl -- provides the expected mobile interaction pattern.

## What Changes

- Add a pure-logic module (`ui/src/lib/pull-to-refresh.ts`) containing all threshold constants, elastic offset calculations, capability detection, route eligibility, overlay blocking, and state derivation as side-effect-free functions.
- Add capability detection: standalone PWA check (`display-mode: standalone` media query + `navigator.standalone` for iOS) and touch device check (`pointer: coarse`, `maxTouchPoints`, `ontouchstart`).
- Add eligibility gating: pull-to-refresh only activates when standalone + touch-capable + allowed route + scrolled to top + no open `<dialog>` overlay.
- Wire touch gesture handlers (`touchstart`, `touchmove`, `touchend`, `touchcancel`) in the root layout with non-passive `touchmove` to block native scroll during the pull gesture.
- Implement elastic pull offset with exponential decay past the 128px threshold for natural feel, with separate offset tracks for the indicator position (caps at 140px) and main content displacement (wider elastic range).
- Implement a 4-state indicator FSM: `idle` -> `pulling` -> `release` -> `refreshing`. Fixed-position indicator slides down from above the viewport with a CSS-only spinner and check icon.
- Push main content down via `margin-top` during the gesture, with smooth snap-back transition (`0.18s ease`) only applied when the finger lifts (no transition during active drag).
- Execute `window.location.reload()` after a 120ms delay to let the refreshing spinner render before the reload blanks the page.
- Reset gesture on route navigation (unless actively refreshing).
- Add i18n keys for pull-to-refresh labels ("Pull to refresh", "Release to refresh", "Refreshing...").
- Add comprehensive vitest tests for all pure logic functions.

## Capabilities

### New Capabilities

- `pull-to-refresh`: PWA pull-to-refresh gesture with capability detection, eligibility gating, elastic touch tracking, visual indicator FSM, and page reload. Pure logic module with root layout integration.

### Modified Capabilities

- `app-layout`: Root layout gains touch event handlers, pull indicator markup, and content offset styles for the pull-to-refresh gesture.
- `i18n`: New translation keys for pull-to-refresh labels.

## Impact

- **UI files**: `ui/src/lib/pull-to-refresh.ts` (new), `ui/src/routes/+layout.svelte` (modified), `ui/src/lib/i18n/en.json` and `de.json` (new keys).
- **Tests**: `ui/src/lib/pull-to-refresh.test.ts` (new, comprehensive unit tests for all pure functions).
- **No backend changes**: This is a frontend-only feature.
- **No new dependencies**: Uses only native browser APIs and existing Svelte 5 runes.
- **No breaking changes**: Feature is gated behind standalone PWA + touch detection; browsers and non-PWA usage are unaffected.
