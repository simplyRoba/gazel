## Purpose

Defines pull-to-refresh eligibility, gesture behavior, visual feedback, content movement, and page reload behavior for the installed application.

## Requirements

### Requirement: Pull-to-refresh availability

Pull-to-refresh SHALL be available only in an installed standalone session on a touch-capable device and an eligible route.

#### Scenario: Installed touch session
- **WHEN** Gazel is running as an installed standalone application on a touch-capable device
- **AND** the current route is eligible
- **THEN** pull-to-refresh MAY begin when its other gesture conditions are met

#### Scenario: Browser tab or non-touch device
- **WHEN** Gazel is running in a regular browser tab or the device is not touch-capable
- **THEN** pull-to-refresh SHALL NOT begin

#### Scenario: Eligible routes
- **WHEN** the current path is `/` or `/settings`
- **THEN** the route SHALL be eligible for pull-to-refresh

#### Scenario: Ineligible routes
- **WHEN** the current path is any route other than `/` or `/settings`
- **THEN** pull-to-refresh SHALL NOT begin

### Requirement: Pull-to-refresh safety conditions

A gesture SHALL begin only from the top of the applicable scroll area, with no modal open, and with exactly one touch point.

#### Scenario: All conditions are met
- **WHEN** the session and route are eligible
- **AND** the page and touched scroll area are at the top
- **AND** no modal is open
- **AND** exactly one touch point starts a downward gesture
- **THEN** pull-to-refresh SHALL begin

#### Scenario: Modal is open
- **WHEN** any modal is open
- **THEN** pull-to-refresh SHALL NOT begin

#### Scenario: Multiple touch points
- **WHEN** more than one touch point is present at gesture start or during movement
- **THEN** the pull-to-refresh gesture SHALL be rejected or reset

#### Scenario: Touched content is scrolled
- **WHEN** the relevant scroll area under the initial touch is below its top position at gesture start or during movement
- **THEN** pull-to-refresh SHALL be rejected or reset
- **AND** reaching the top during that touch sequence SHALL NOT arm refresh
- **AND** a new eligible single-touch gesture SHALL be required

### Requirement: Pull states and activation threshold

The gesture SHALL move through idle, pulling, release, and refreshing states using a 128px activation threshold.

#### Scenario: Idle
- **WHEN** no gesture is active or refreshing
- **AND** there is no positive downward pull
- **THEN** the gesture SHALL remain idle

#### Scenario: Pulling below threshold
- **WHEN** the downward pull is greater than zero and less than 128px
- **THEN** the gesture SHALL be in the pulling state

#### Scenario: Release threshold reached
- **WHEN** the downward pull reaches or exceeds 128px
- **THEN** the gesture SHALL be armed in the release state

#### Scenario: Finger released above threshold
- **WHEN** the user releases an armed gesture
- **THEN** the state SHALL become refreshing
- **AND** the page reload SHALL begin after 120ms

#### Scenario: Finger released below threshold
- **WHEN** the user releases a gesture below 128px
- **THEN** the gesture SHALL settle back to idle without reloading

#### Scenario: Gesture is canceled
- **WHEN** the active touch sequence is canceled
- **THEN** the gesture SHALL reset to idle

### Requirement: Elastic pull feedback

The indicator and page content SHALL follow the user's downward pull directly until the activation threshold, then continue with increasing resistance.

#### Scenario: Pull below threshold
- **WHEN** the downward pull is between zero and 128px
- **THEN** the indicator and content displacement SHALL track the pull distance directly

#### Scenario: Pull above threshold
- **WHEN** the downward pull exceeds 128px
- **THEN** the indicator SHALL continue moving with increasing resistance toward, but not beyond, 140px from its resting position
- **AND** the content SHALL continue moving with gentler resistance toward, but not beyond, 228px from its resting position
- **AND** content displacement SHALL remain greater than indicator displacement above the threshold

#### Scenario: Upward or zero movement
- **WHEN** movement is upward or zero
- **THEN** the indicator and content SHALL remain at their resting positions

#### Scenario: Native scrolling during active pull
- **WHEN** an eligible downward pull is actively moving the indicator and content
- **THEN** native page scrolling SHALL NOT move the page independently of the gesture

### Requirement: Pull indicator feedback

A pull indicator SHALL appear at the top of the viewport only while a pull-to-refresh gesture is active.

#### Scenario: Idle indicator
- **WHEN** the gesture is idle
- **THEN** the indicator SHALL not be visible

#### Scenario: Pulling indicator
- **WHEN** the gesture is pulling below the threshold
- **THEN** a circular spinner SHALL be displayed
- **AND** its rotation SHALL increase progressively from zero to one full turn as the pull approaches 128px

#### Scenario: Armed indicator
- **WHEN** the gesture reaches the release state
- **THEN** a check icon SHALL replace the spinner

#### Scenario: Refreshing indicator
- **WHEN** the gesture is refreshing
- **THEN** the spinner SHALL rotate continuously until the reload completes

### Requirement: Content settling

Page content SHALL track active touch movement immediately and settle smoothly after a non-refreshing gesture ends.

#### Scenario: Active movement
- **WHEN** the user moves an active pull gesture
- **THEN** content displacement SHALL update without transition lag

#### Scenario: Gesture settles
- **WHEN** a gesture ends without entering the refreshing state
- **THEN** content SHALL animate smoothly back to its resting position

### Requirement: Route navigation resets gestures

Navigation SHALL reset an active gesture unless a page reload is already in progress.

#### Scenario: Navigation during pull
- **WHEN** the route changes while the gesture is not refreshing
- **THEN** the gesture SHALL reset to idle

#### Scenario: Navigation during refresh
- **WHEN** the route changes while the gesture is refreshing
- **THEN** the refreshing state SHALL remain until the reload completes
