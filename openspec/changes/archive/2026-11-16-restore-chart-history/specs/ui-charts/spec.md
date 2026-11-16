# Spec Delta

## MODIFIED Requirements

### Requirement: Compact efficiency trend

The dashboard SHALL display the exact all-time average efficiency with a low-contrast background trend from the statistics response. The trend SHALL use at most 60 representative eligible values; the displayed average SHALL NOT be calculated from that reduced series.

#### Scenario: Efficiency summary data
- **WHEN** the complete eligible history contains valid efficiency segments
- **THEN** the card SHALL display the all-time average formatted with the user's response units
- **AND** segments where `is_valid` is `false` SHALL be excluded
- **AND** a chronological line-and-area trend SHALL appear when at least two valid segments exist

#### Scenario: Insufficient efficiency history
- **WHEN** fewer than two valid segments exist
- **THEN** no efficiency background trend SHALL be displayed

#### Scenario: Long history
- **WHEN** more than 60 valid segment values exist
- **THEN** the trend SHALL use the supplied representative values and original relative positions
- **AND** its displayed average SHALL still include every valid segment

### Requirement: Distance trend chart

The dashboard SHALL display the selected vehicle's server-prepared monthly/yearly distance trend as a line-and-area chart. Monthly mode SHALL show a navigable window of at most 12 calendar months; yearly mode SHALL show the complete yearly series. Existing calendar allocation, distance formatting, zero-based value domain, and eligible-point thresholds SHALL remain unchanged.

#### Scenario: Monthly and yearly distance
- **WHEN** at least two eligible period points exist in the selected view
- **THEN** the distance chart SHALL plot that view and allow switching modes
- **AND** axes and point details SHALL use the response's distance unit
- **AND** the value-axis domain SHALL start at zero

#### Scenario: Older periods omitted
- **WHEN** distance history contains periods outside the initially visible latest 12-month window
- **THEN** previous/next controls SHALL make every older applicable month reachable without rendering all months at once
- **AND** the chart SHALL show its visible date range without an older-period omission caption

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
- **THEN** previous/next controls SHALL make every older applicable month reachable without rendering all months at once
- **AND** the chart SHALL show its visible date range without an older-period omission caption
- **AND** all-time summary totals SHALL remain unaffected

#### Scenario: Complete yearly cost
- **WHEN** cost history exceeds 12 years
- **THEN** yearly mode SHALL show the earliest year, latest year, and every intervening year without a 12-year cutoff

### Requirement: Fuel price trend chart

The dashboard SHALL display the selected vehicle's complete server-prepared eligible fuel-price history on one continuous timeline, independently of paginated fill-up entries. It SHALL plot one eligible recorded price as a visible dot and two or more prices as the existing line/trend presentation. It SHALL NOT impose a point-count cap, sample observations, or apply cost/distance period windows to this chart.

#### Scenario: Fuel price data
- **WHEN** at least one eligible recorded price exists
- **THEN** the title SHALL identify the response currency and volume unit, such as `USD/L`
- **AND** axis values SHALL omit the repeated volume suffix while point details SHALL include it
- **AND** the time axis SHALL show recorded fill-up dates
- **AND** a single point SHALL be a visible dot with point details available
- **AND** the first eligible partial fill SHALL NOT require a full-tank segment

#### Scenario: No eligible fuel-price data
- **WHEN** no eligible recorded prices exist
- **THEN** the chart SHALL show the existing more-fill-ups-needed message

#### Scenario: Different prices on the same date
- **WHEN** multiple eligible recorded prices share a date but have different values
- **THEN** hover or touch selection at that date SHALL allow each distinct recorded value to be inspected rather than always selecting the first observation

#### Scenario: Reduced fuel-price history
- **WHEN** more than 240 eligible recorded prices exist
- **THEN** all recorded observations SHALL remain in the plotted series across the complete time domain
- **AND** point details SHALL describe actual recorded prices, not fabricated averages
- **AND** no reduced-history caption or monthly/yearly navigation SHALL appear
- **AND** the supplied trend SHALL represent the complete eligible price population

### Requirement: Chart calculation rules

Full-size charts SHALL use complete server-prepared period aggregates and recorded prices; summary sparklines SHALL remain bounded. Efficiency, cost, and distance SHALL retain their existing segment-based calculations; fuel prices SHALL use individual eligible fills. The client SHALL format, select visible monthly windows, and render these values without recalculating aggregates or deriving chart data from list pages.

#### Scenario: Efficiency trend calculation
- **WHEN** efficiency trend data is prepared
- **THEN** only valid segments SHALL contribute
- **AND** values SHALL remain chronologically ordered by segment end date

#### Scenario: Monthly cost calculation
- **WHEN** period cost data is prepared
- **THEN** segment costs SHALL be summed into their ending calendar periods and ordered chronologically

#### Scenario: Monthly and yearly distance calculation
- **WHEN** period distance data is prepared
- **THEN** each segment's distance SHALL be distributed proportionally by calendar days across the periods it spans
- **AND** period totals SHALL remain chronological and use the existing integer-rounding rule

#### Scenario: Fuel-price calculation
- **WHEN** recorded prices are prepared
- **THEN** each eligible price SHALL equal cost divided by fuel volume normalized to the response's volume unit
- **AND** nonpositive or nonfinite fuel and missing or nonfinite cost SHALL be excluded, while zero cost SHALL be included
- **AND** partial and missed fills SHALL remain eligible regardless of odometer or segment availability
- **AND** every eligible observation SHALL be preserved in date-then-ID order

#### Scenario: Compact trend ordering
- **WHEN** a background trend is prepared
- **THEN** it SHALL preserve the original relative positions of retained chronological values
- **AND** the cost-per-distance trend SHALL use the selected vehicle's segment values, including invalid efficiency segments, with at most 60 retained values

#### Scenario: List pagination
- **WHEN** older fill-up entries load on scroll
- **THEN** rendered chart data, trend lines, axes, and sparklines SHALL remain unchanged

## ADDED Requirements

### Requirement: Monthly chart window navigation

Monthly cost and distance charts SHALL provide keyboard- and touch-operable previous/next navigation through their full calendar history, with a concise visible date range and localized accessible control names. Navigation SHALL change only the visible chart window, not full-history summaries or the fuel-price timeline. Omission-warning captions SHALL NOT be displayed.

#### Scenario: Previous and next windows
- **WHEN** a user navigates backward or forward in monthly cost or distance
- **THEN** the chart SHALL display at most 12 consecutive calendar months from its complete history and update the visible date range
- **AND** repeated navigation SHALL reach both the earliest and latest applicable months
- **AND** no statistics or fill-up-list request SHALL be required solely for navigation

#### Scenario: Navigation endpoints
- **WHEN** the visible window includes the earliest applicable month
- **THEN** backward navigation SHALL be disabled
- **WHEN** the visible window includes the latest applicable month
- **THEN** forward navigation SHALL be disabled
- **AND** navigation SHALL NOT create empty windows outside the source span

#### Scenario: Short and oldest windows
- **WHEN** a complete monthly history has at most 12 months
- **THEN** all of its months SHALL be visible and navigation controls SHALL be hidden
- **WHEN** a longer history reaches its oldest window
- **THEN** that window SHALL include up to 12 available months, overlapping the adjacent window if needed rather than isolating a single oldest month
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

#### Scenario: Responsive navigation
- **WHEN** a monthly chart appears in the compact carousel or desktop panel
- **THEN** its navigation and date range SHALL fit within the card without horizontal overflow
- **AND** its controls SHALL remain operable without relying solely on chart-swipe gestures
