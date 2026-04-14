## 1. Pure logic module

- [x] 1.1 Create `ui/src/lib/pull-to-refresh.ts` with constants: `PULL_TO_REFRESH_THRESHOLD` (128), `MAX_PULL_TO_REFRESH_OFFSET` (140), `PULL_TO_REFRESH_RELOAD_DELAY_MS` (120)
- [x] 1.2 Implement `isStandalonePwaSession(window)` — detect standalone PWA via `display-mode: standalone` media query and `navigator.standalone` (iOS)
- [x] 1.3 Implement `isTouchCapableDevice(window)` — detect touch via `pointer: coarse`, `maxTouchPoints > 0`, `'ontouchstart' in window`
- [x] 1.4 Implement `isPullToRefreshRoute(pathname)` — allow `/` and `/settings`, block `/settings/vehicles/new`, `/settings/vehicles/*/edit`, and everything else
- [x] 1.5 Implement `hasBlockingPullToRefreshOverlay(document)` — returns `true` when `document.querySelector('dialog[open]')` finds an element
- [x] 1.6 Implement `canStartPullToRefresh(params)` — gate combining standalone + touch + route + scrollTop <= 0 + no overlay
- [x] 1.7 Implement `getPullIndicatorState(rawPullDistance)` and export `PullIndicatorState` type — returns `"idle"` / `"pulling"` / `"release"` based on distance vs threshold
- [x] 1.8 Implement `calculatePullOffset(distance)` — linear up to threshold, then exponential decay with 12px elastic range (max 140px)
- [x] 1.9 Implement `calculateContentOffset(distance)` — linear up to threshold, then exponential decay with 100px elastic range
- [x] 1.10 Implement `shouldTriggerPullToRefresh(rawPullDistance)` — returns `true` when distance >= threshold
- [x] 1.11 Implement `schedulePullToRefreshReload(window, reloadFn, delayMs)` — schedules reload via `setTimeout`, returns timeout ID

## 2. Unit tests

- [x] 2.1 Create `ui/src/lib/pull-to-refresh.test.ts` with tests for `isStandalonePwaSession` (standalone media match, navigator.standalone, browser tab)
- [x] 2.2 Add tests for `isTouchCapableDevice` (coarse pointer, maxTouchPoints, ontouchstart, no touch)
- [x] 2.3 Add tests for `isPullToRefreshRoute` (allowed routes, blocked form routes, unknown routes)
- [x] 2.4 Add tests for `hasBlockingPullToRefreshOverlay` (dialog open, no dialog)
- [x] 2.5 Add tests for `canStartPullToRefresh` (all conditions met, each condition failing individually)
- [x] 2.6 Add tests for `getPullIndicatorState` (idle/pulling/release boundaries, negative distance)
- [x] 2.7 Add tests for `calculatePullOffset` (zero, linear range, above threshold approaches max, large values)
- [x] 2.8 Add tests for `calculateContentOffset` (zero, linear range, above threshold with wider elastic range)
- [x] 2.9 Add tests for `shouldTriggerPullToRefresh` (below/at/above threshold)
- [x] 2.10 Add tests for `schedulePullToRefreshReload` (calls reload after delay, returns timeout ID)

## 3. i18n keys

- [x] 3.1 Add `pullToRefresh.pulling`, `pullToRefresh.release`, `pullToRefresh.refreshing` keys to `ui/src/lib/i18n/en.json`
- [x] 3.2 Add corresponding German translations to `ui/src/lib/i18n/de.json`

## 4. Root layout integration

- [x] 4.1 Add imports and capability detection reactive state in `+layout.svelte` — compute `canUsePullToRefresh` from `isStandalonePwaSession` and `isTouchCapableDevice` on mount
- [x] 4.2 Wire touch event handlers on window (`touchstart` passive, `touchmove` non-passive, `touchend`, `touchcancel`) with gesture state tracking (`gestureActive`, `touchStartY`, `rawPullDistance`, `pullOffset`, `contentOffset`, `pullIndicatorState`)
- [x] 4.3 Implement `touchstart` handler — reject multi-touch, check eligibility via `canStartPullToRefresh`, record start Y, clear pending reload timeout
- [x] 4.4 Implement `touchmove` handler — reject multi-touch or mid-gesture overlay, calculate distance, update offsets via `calculatePullOffset`/`calculateContentOffset`, derive state via `getPullIndicatorState`, call `preventDefault` when pulling
- [x] 4.5 Implement `touchend` handler — if `shouldTriggerPullToRefresh`, set state to `"refreshing"` and call `schedulePullToRefreshReload`; otherwise reset gesture
- [x] 4.6 Implement `touchcancel` handler — always reset gesture
- [x] 4.7 Add `$effect` for route change gesture reset (reset unless state is `"refreshing"`)
- [x] 4.8 Add `$effect` for capability change gesture reset

## 5. Pull indicator markup and styles

- [x] 5.1 Add pull indicator fixed `<div>` in `+layout.svelte` — positioned at top of viewport, hidden when idle, with `settling` class for snap-back transition
- [x] 5.2 Add CSS-only spinner element (border trick, `border-top-color` accent, `border-radius: 999px`) with proportional rotation during pulling and infinite animation during refreshing
- [x] 5.3 Add check icon display when state is `"release"` (Lucide Check or inline SVG)
- [x] 5.4 Add i18n label display (`t('pullToRefresh.pulling')`, `t('pullToRefresh.release')`, `t('pullToRefresh.refreshing')`)
- [x] 5.5 Apply `margin-top` style to `<main>` content area driven by `contentOffset`, with `settling` class for `0.18s ease` transition only when finger is lifted
- [x] 5.6 Add `@keyframes pull-refresh-spin` animation and all pull indicator CSS rules (z-index 160, flex column, pointer-events none)

## 6. Verification

- [x] 6.1 Run `npm run test --prefix ui` — all pull-to-refresh tests and i18n completeness tests pass
- [x] 6.2 Run `npm run check --prefix ui` — TypeScript type checking passes
- [x] 6.3 Run `npm run format:check --prefix ui` and `npm run lint --prefix ui` — formatting and lint pass
- [x] 6.4 Run `cargo fmt -- --check` and `cargo clippy -- -D warnings` — Rust checks pass (no backend changes but verify no regressions)
- [x] 6.5 Run `cargo test` — full test suite passes
