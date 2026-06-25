## Purpose

Defines the pull-to-refresh gesture system for the PWA, including pure logic functions, touch gesture handling, visual indicator behavior, and content displacement during the gesture.

## Requirements

### Requirement: Pure logic module

The app SHALL provide a pure logic module at `ui/src/lib/pull-to-refresh.ts` containing all pull-to-refresh constants, calculations, and state derivation as side-effect-free exported functions. The module SHALL have no DOM, framework, or browser API dependencies.

#### Scenario: Module exports constants
- **WHEN** the module is imported
- **THEN** it SHALL export `PULL_TO_REFRESH_THRESHOLD` as `128` (pixels)
- **AND** it SHALL export `MAX_PULL_TO_REFRESH_OFFSET` as `140` (pixels)
- **AND** it SHALL export `PULL_TO_REFRESH_RELOAD_DELAY_MS` as `120` (milliseconds)

### Requirement: Standalone PWA detection

The module SHALL export an `isStandalonePwaSession(window)` function that returns `true` only when the app is running as an installed standalone PWA.

#### Scenario: Standalone via media query
- **WHEN** `window.matchMedia('(display-mode: standalone)')` matches
- **THEN** `isStandalonePwaSession()` SHALL return `true`

#### Scenario: Standalone via iOS navigator property
- **WHEN** `navigator.standalone` is `true` (iOS Safari)
- **THEN** `isStandalonePwaSession()` SHALL return `true`

#### Scenario: Running in browser tab
- **WHEN** neither the media query matches nor `navigator.standalone` is true
- **THEN** `isStandalonePwaSession()` SHALL return `false`

### Requirement: Touch capability detection

The module SHALL export an `isTouchCapableDevice(window)` function that returns `true` when the device supports touch input.

#### Scenario: Coarse pointer detected
- **WHEN** `window.matchMedia('(pointer: coarse)')` matches
- **THEN** `isTouchCapableDevice()` SHALL return `true`

#### Scenario: Touch points detected
- **WHEN** `navigator.maxTouchPoints` is greater than `0`
- **THEN** `isTouchCapableDevice()` SHALL return `true`

#### Scenario: ontouchstart in window
- **WHEN** `'ontouchstart' in window` is `true`
- **THEN** `isTouchCapableDevice()` SHALL return `true`

#### Scenario: No touch support
- **WHEN** none of the above conditions are met
- **THEN** `isTouchCapableDevice()` SHALL return `false`

### Requirement: Route eligibility

The module SHALL export an `isPullToRefreshRoute(pathname)` function that returns `true` only for routes where pull-to-refresh is allowed.

#### Scenario: Dashboard route allowed
- **WHEN** pathname is `/`
- **THEN** `isPullToRefreshRoute()` SHALL return `true`

#### Scenario: Settings route allowed
- **WHEN** pathname is `/settings`
- **THEN** `isPullToRefreshRoute()` SHALL return `true`

#### Scenario: Vehicle form routes blocked
- **WHEN** pathname is `/settings/vehicles/new` or `/settings/vehicles/{id}/edit`
- **THEN** `isPullToRefreshRoute()` SHALL return `false`

#### Scenario: Unknown routes blocked
- **WHEN** pathname does not match any allowed route pattern
- **THEN** `isPullToRefreshRoute()` SHALL return `false`

### Requirement: Overlay blocking detection

The module SHALL export a `hasBlockingPullToRefreshOverlay(document)` function that returns `true` when any modal dialog is open.

#### Scenario: Open dialog blocks pull
- **WHEN** `document.querySelector('dialog[open]')` returns an element
- **THEN** `hasBlockingPullToRefreshOverlay()` SHALL return `true`

#### Scenario: No blocking overlay
- **WHEN** no `dialog[open]` element exists in the DOM
- **THEN** `hasBlockingPullToRefreshOverlay()` SHALL return `false`

### Requirement: Pull-to-refresh eligibility gate

The module SHALL export a `canStartPullToRefresh(params)` function that returns `true` only when all preconditions are met simultaneously.

#### Scenario: All conditions met
- **WHEN** the app is a standalone PWA AND the device is touch-capable AND the current route is eligible AND the page is scrolled to the top (scrollTop <= 0) AND no dialog overlay is open
- **THEN** `canStartPullToRefresh()` SHALL return `true`

#### Scenario: Any condition fails
- **WHEN** any one of the preconditions is not met
- **THEN** `canStartPullToRefresh()` SHALL return `false`

### Requirement: Pull indicator state derivation

The module SHALL export a `getPullIndicatorState(rawPullDistance)` function that returns a state string based on the pull distance, and a `PullIndicatorState` type.

#### Scenario: Idle state
- **WHEN** rawPullDistance is `0` or negative
- **THEN** `getPullIndicatorState()` SHALL return `"idle"`

#### Scenario: Pulling state
- **WHEN** rawPullDistance is greater than `0` and less than `PULL_TO_REFRESH_THRESHOLD` (128)
- **THEN** `getPullIndicatorState()` SHALL return `"pulling"`

#### Scenario: Release state
- **WHEN** rawPullDistance is greater than or equal to `PULL_TO_REFRESH_THRESHOLD` (128)
- **THEN** `getPullIndicatorState()` SHALL return `"release"`

### Requirement: Elastic pull offset calculation

The module SHALL export a `calculatePullOffset(distance)` function that returns the indicator's visual offset with exponential decay past the threshold.

#### Scenario: Below threshold
- **WHEN** distance is between `0` and `PULL_TO_REFRESH_THRESHOLD` (128)
- **THEN** `calculatePullOffset()` SHALL return the distance unchanged (linear 1:1 tracking)

#### Scenario: Above threshold with elastic decay
- **WHEN** distance exceeds `PULL_TO_REFRESH_THRESHOLD`
- **THEN** `calculatePullOffset()` SHALL return a value between `PULL_TO_REFRESH_THRESHOLD` and `MAX_PULL_TO_REFRESH_OFFSET` (140)
- **AND** the value SHALL use the formula `THRESHOLD + elasticRange * (1 - exp(-overThreshold / elasticRange))` where `elasticRange = MAX_OFFSET - THRESHOLD`

#### Scenario: Zero or negative distance
- **WHEN** distance is `0` or negative
- **THEN** `calculatePullOffset()` SHALL return `0`

### Requirement: Elastic content offset calculation

The module SHALL export a `calculateContentOffset(distance)` function that returns the main content's vertical displacement with a wider elastic range than the indicator.

#### Scenario: Below threshold
- **WHEN** distance is between `0` and `PULL_TO_REFRESH_THRESHOLD`
- **THEN** `calculateContentOffset()` SHALL return the distance unchanged (linear 1:1 tracking)

#### Scenario: Above threshold with wide elastic decay
- **WHEN** distance exceeds `PULL_TO_REFRESH_THRESHOLD`
- **THEN** `calculateContentOffset()` SHALL use an elastic range of `100` pixels (wider than the indicator's range)
- **AND** the value SHALL asymptotically approach `THRESHOLD + 100`

#### Scenario: Zero or negative distance
- **WHEN** distance is `0` or negative
- **THEN** `calculateContentOffset()` SHALL return `0`

### Requirement: Refresh trigger check

The module SHALL export a `shouldTriggerPullToRefresh(rawPullDistance)` function.

#### Scenario: Distance meets threshold
- **WHEN** rawPullDistance is greater than or equal to `PULL_TO_REFRESH_THRESHOLD`
- **THEN** `shouldTriggerPullToRefresh()` SHALL return `true`

#### Scenario: Distance below threshold
- **WHEN** rawPullDistance is less than `PULL_TO_REFRESH_THRESHOLD`
- **THEN** `shouldTriggerPullToRefresh()` SHALL return `false`

### Requirement: Reload scheduling

The module SHALL export a `schedulePullToRefreshReload(window, reloadFn, delayMs)` function that schedules a reload after a configurable delay, returning a timeout ID for cancellation.

#### Scenario: Reload called after delay
- **WHEN** `schedulePullToRefreshReload()` is called with a 120ms delay
- **THEN** the provided `reloadFn` SHALL be called after 120 milliseconds
- **AND** the function SHALL return a timeout ID

### Requirement: Touch gesture handling in root layout

The root layout SHALL register touch event handlers on the window to implement the pull-to-refresh gesture.

#### Scenario: Single-touch gesture start
- **WHEN** a `touchstart` event fires with exactly one touch point AND all eligibility conditions are met
- **THEN** the layout SHALL record the starting Y position and activate the gesture

#### Scenario: Multi-touch rejection
- **WHEN** a `touchstart` or `touchmove` event fires with more than one touch point
- **THEN** the gesture SHALL be rejected or reset

#### Scenario: Touchmove tracks pull distance
- **WHEN** a `touchmove` event fires during an active gesture
- **THEN** the raw pull distance SHALL be calculated as `currentY - startY`
- **AND** `calculatePullOffset()` and `calculateContentOffset()` SHALL update the indicator and content positions
- **AND** `event.preventDefault()` SHALL be called to block native scroll when pull distance is positive

#### Scenario: Touchend triggers refresh
- **WHEN** a `touchend` event fires during an active gesture AND `shouldTriggerPullToRefresh()` returns true
- **THEN** the indicator state SHALL transition to `"refreshing"` and a reload SHALL be scheduled

#### Scenario: Touchend cancels gesture
- **WHEN** a `touchend` event fires during an active gesture AND the pull distance is below threshold
- **THEN** the gesture SHALL reset to idle with a settling transition

#### Scenario: Touchcancel resets gesture
- **WHEN** a `touchcancel` event fires
- **THEN** the gesture SHALL always reset to idle

### Requirement: Pull indicator visual element

The root layout SHALL render a fixed-position pull indicator element at the top of the viewport.

#### Scenario: Indicator hidden when idle
- **WHEN** the pull indicator state is `"idle"`
- **THEN** the indicator element SHALL not be visible

#### Scenario: Spinner rotation during pulling
- **WHEN** the pull indicator state is `"pulling"`
- **THEN** a CSS-only circular spinner SHALL be displayed
- **AND** the spinner rotation SHALL be proportional to `rawPullDistance / THRESHOLD * 360` degrees

#### Scenario: Check icon on release
- **WHEN** the pull indicator state is `"release"`
- **THEN** a check icon SHALL replace the spinner to indicate the gesture is armed

#### Scenario: Infinite spinner during refreshing
- **WHEN** the pull indicator state is `"refreshing"`
- **THEN** the spinner SHALL animate with an infinite rotation (`0.8s linear infinite`)

### Requirement: Content push during gesture

The main content area SHALL shift down via `margin-top` during the pull gesture.

#### Scenario: Content follows pull
- **WHEN** the gesture is active and pull distance is positive
- **THEN** the main content's `margin-top` SHALL be set to `calculateContentOffset(distance)` pixels

#### Scenario: Settling transition on release
- **WHEN** the gesture ends (finger lifts) and the state is not `"refreshing"`
- **THEN** a CSS transition (`0.18s ease`) SHALL animate the content back to zero margin-top

#### Scenario: No transition during active drag
- **WHEN** the gesture is actively tracking touch movement
- **THEN** no CSS transition SHALL be applied to the content margin-top (instant tracking)

### Requirement: Gesture reset on route navigation

The gesture SHALL reset when the user navigates to a different route.

#### Scenario: Navigation resets active gesture
- **WHEN** the route pathname changes AND the indicator state is not `"refreshing"`
- **THEN** the gesture SHALL reset to idle

#### Scenario: Navigation during refresh does not reset
- **WHEN** the route pathname changes AND the indicator state is `"refreshing"`
- **THEN** the gesture SHALL NOT reset (the reload will complete)

### Requirement: Pull-to-refresh unit tests

The module SHALL have comprehensive vitest tests for all exported pure functions.

#### Scenario: All pure functions tested
- **WHEN** vitest runs on `pull-to-refresh.test.ts`
- **THEN** tests SHALL cover: `isStandalonePwaSession`, `isTouchCapableDevice`, `isPullToRefreshRoute`, `hasBlockingPullToRefreshOverlay`, `canStartPullToRefresh`, `getPullIndicatorState`, `calculatePullOffset`, `calculateContentOffset`, `shouldTriggerPullToRefresh`, and `schedulePullToRefreshReload`
