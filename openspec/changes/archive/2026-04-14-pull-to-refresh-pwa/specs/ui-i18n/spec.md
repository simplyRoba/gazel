## ADDED Requirements

### Requirement: Pull-to-refresh translation keys

The translation files SHALL include keys for pull-to-refresh indicator labels.

#### Scenario: English pull-to-refresh keys
- **WHEN** `en.json` is loaded
- **THEN** it SHALL contain `"pullToRefresh.pulling"` with value `"Pull to refresh"`
- **AND** it SHALL contain `"pullToRefresh.release"` with value `"Release to refresh"`
- **AND** it SHALL contain `"pullToRefresh.refreshing"` with value `"Refreshing..."`

#### Scenario: German pull-to-refresh keys
- **WHEN** `de.json` is loaded
- **THEN** it SHALL contain `"pullToRefresh.pulling"` with the German translation
- **AND** it SHALL contain `"pullToRefresh.release"` with the German translation
- **AND** it SHALL contain `"pullToRefresh.refreshing"` with the German translation
