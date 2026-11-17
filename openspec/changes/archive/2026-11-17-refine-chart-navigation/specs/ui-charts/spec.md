# Spec Delta

## MODIFIED Requirements

### Requirement: Distance trend chart

The dashboard SHALL display the selected vehicle's server-prepared monthly/yearly distance trend as a line-and-area chart. Monthly mode SHALL show a navigable window of at most 12 calendar months; yearly mode SHALL show the complete yearly series. Existing calendar allocation, distance formatting, zero-based value domain, and eligible-point thresholds SHALL remain unchanged.

#### Scenario: Monthly and yearly distance
- **WHEN** at least two eligible period points exist in the selected view
- **THEN** the distance chart SHALL plot that view and allow switching modes
- **AND** axes and point details SHALL use the response's distance unit
- **AND** the value-axis domain SHALL start at zero

#### Scenario: Older periods omitted
- **WHEN** distance history contains periods outside the initially visible latest 12-month window
- **THEN** in the desktop layout, previous/next controls SHALL make every older applicable month reachable without rendering all months at once
- **AND** the chart SHALL NOT show an older-period omission caption or a visible date range

#### Scenario: Complete yearly distance
- **WHEN** distance history exceeds 12 years
- **THEN** yearly mode SHALL show the earliest year, latest year, and every intervening year without a 12-year cutoff

### Requirement: Monthly cost trend chart

The dashboard SHALL display server-prepared calendar-period cost totals as bars for the selected vehicle. Monthly mode SHALL show a navigable window of at most 12 calendar months; yearly mode SHALL show the complete yearly series without changing existing segment-cost allocation or currency formatting.

#### Scenario: Monthly cost data
- **WHEN** eligible segment costs exist
- **THEN** each bar SHALL represent a calendar-period total
- **AND** the category axis SHALL show period labels
- **AND** the value axis SHALL use the response's currency

#### Scenario: Older periods omitted
- **WHEN** cost history contains periods outside the initially visible latest 12-month window
- **THEN** in the desktop layout, previous/next controls SHALL make every older applicable month reachable without rendering all months at once
- **AND** the chart SHALL NOT show an older-period omission caption or a visible date range
- **AND** all-time summary totals SHALL remain unaffected

#### Scenario: Complete yearly cost
- **WHEN** cost history exceeds 12 years
- **THEN** yearly mode SHALL show the earliest year, latest year, and every intervening year without a 12-year cutoff

### Requirement: Monthly chart window navigation

In the desktop layout, monthly cost and distance charts SHALL provide keyboard-operable controls that move the visible 12-month window backward or forward by one month or by twelve months through their full calendar history, with localized accessible control names. The controls SHALL be visually consistent with the monthly/yearly mode toggle. The charts SHALL NOT display a visible date range or omission-warning captions; the category axis identifies the visible months. Navigation SHALL change only the visible chart window, not full-history summaries or the fuel-price timeline.

#### Scenario: Single-month steps
- **WHEN** a user moves backward or forward by one month in monthly cost or distance
- **THEN** the chart SHALL display at most 12 consecutive calendar months shifted by one month
- **AND** any 12 consecutive applicable months, such as January through December of one year, SHALL be reachable regardless of the latest recorded month
- **AND** no statistics or fill-up-list request SHALL be required solely for navigation

#### Scenario: Twelve-month steps
- **WHEN** a user moves backward or forward by twelve months
- **THEN** the chart SHALL display the same calendar months of the previous or next year where available
- **AND** a step that would pass the earliest or latest applicable window SHALL stop at that window

#### Scenario: Navigation endpoints
- **WHEN** the visible window includes the earliest applicable month
- **THEN** all backward controls SHALL be disabled
- **WHEN** the visible window includes the latest applicable month
- **THEN** all forward controls SHALL be disabled
- **AND** navigation SHALL NOT create empty windows outside the source span

#### Scenario: Short and oldest windows
- **WHEN** a complete monthly history has at most 12 months
- **THEN** all of its months SHALL be visible and navigation controls SHALL be hidden
- **WHEN** a longer history reaches its oldest window
- **THEN** that window SHALL include up to 12 available months rather than isolating a single oldest month
- **AND** existing cost and distance insufficient-data rules SHALL apply to genuinely short histories

#### Scenario: Mode switching
- **WHEN** a user switches to yearly mode
- **THEN** monthly navigation SHALL be hidden and all yearly points SHALL be displayed
- **WHEN** the user switches back to monthly mode for the same vehicle
- **THEN** the previously selected monthly window SHALL be restored

#### Scenario: Selection and refresh
- **WHEN** the selected vehicle changes
- **THEN** each monthly chart SHALL start at the new vehicle's latest applicable window
- **WHEN** data refreshes for the same vehicle while the latest window is selected
- **THEN** the chart SHALL continue to show the latest window
- **WHEN** data refreshes while an older window is selected
- **THEN** its visible calendar range SHALL be retained where it still exists, or clamped to the nearest available window without becoming empty

#### Scenario: Compact layout
- **WHEN** a monthly chart appears in the compact carousel layout
- **THEN** no monthly navigation controls SHALL be shown
- **AND** the chart SHALL show its current window, which is the latest applicable window unless an older one was selected earlier in the session
- **AND** the chart header SHALL fit within the card without horizontal overflow

#### Scenario: Desktop layout
- **WHEN** a monthly chart appears in the desktop panel
- **THEN** its navigation controls and mode toggle SHALL fit within the card header without horizontal overflow
