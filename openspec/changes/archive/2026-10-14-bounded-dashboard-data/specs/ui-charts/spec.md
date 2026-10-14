# Spec Delta

## MODIFIED Requirements

### Requirement: Compact efficiency trend

The dashboard SHALL display the exact all-time average efficiency with a low-contrast background trend from the bounded dashboard response. The trend SHALL use at most 60 representative eligible values; the displayed average SHALL NOT be calculated from that reduced series.

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

The dashboard SHALL display the selected vehicle's server-prepared monthly/yearly distance trend as a line-and-area chart. Each mode SHALL show at most its latest 12 periods. Existing calendar allocation, distance formatting, and zero-based value-domain rules SHALL remain unchanged.

#### Scenario: Monthly and yearly distance
- **WHEN** at least two eligible period points exist in the selected mode
- **THEN** the distance chart SHALL plot that bounded series and allow switching modes
- **AND** monthly mode SHALL show at most the latest 12 months and yearly mode at most the latest 12 years
- **AND** axes and point details SHALL use the response's distance unit
- **AND** the value-axis domain SHALL start at zero

#### Scenario: Older periods omitted
- **WHEN** history contains periods outside the returned window
- **THEN** the chart SHALL identify its limited displayed range rather than imply that it displays all-time distance

### Requirement: Monthly cost trend chart

The dashboard SHALL display server-prepared calendar-period cost totals as bars for the selected vehicle. Monthly and yearly modes SHALL each show at most the latest 12 periods without changing existing segment-cost allocation or currency formatting.

#### Scenario: Monthly cost data
- **WHEN** eligible segment costs exist
- **THEN** each bar SHALL represent a calendar-period total
- **AND** the category axis SHALL show period labels
- **AND** the value axis SHALL use the response's currency

#### Scenario: Older periods omitted
- **WHEN** history exceeds the period window
- **THEN** the chart SHALL identify its limited displayed range
- **AND** all-time summary totals SHALL remain unaffected

### Requirement: Fuel price trend chart

The dashboard SHALL display the selected vehicle's bounded, server-prepared fuel-price history independently of paginated fill-up entries. It SHALL plot one eligible recorded price as a visible dot and two or more prices as the existing line/trend presentation, with at most 240 retained prices.

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

#### Scenario: Reduced fuel-price history
- **WHEN** more than 240 recorded prices exist
- **THEN** the chart SHALL show the bounded chronological series supplied by the backend
- **AND** retained details SHALL describe actual recorded prices, not fabricated averages
- **AND** the chart SHALL indicate that the history has been reduced
- **AND** the supplied trend SHALL represent the complete eligible price population, not a new regression calculated from retained points alone

### Requirement: Chart calculation rules

Dashboard charts and sparklines SHALL use server-prepared bounded series. Efficiency, cost, and distance SHALL retain their existing segment-based calculations; fuel prices SHALL use individual eligible fills. The client SHALL format and render these values without reconstructing full histories or deriving chart data from list pages.

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
- **AND** date-then-ID order SHALL be preserved before and after reduction

#### Scenario: Compact trend ordering
- **WHEN** a background trend is prepared
- **THEN** it SHALL preserve the original relative positions of retained chronological values
- **AND** the cost-per-distance trend SHALL use the selected vehicle's segment values, including invalid efficiency segments, with at most 60 retained values

#### Scenario: List pagination
- **WHEN** older fill-up entries load on scroll
- **THEN** rendered chart data, trend lines, axes, and sparklines SHALL remain unchanged
