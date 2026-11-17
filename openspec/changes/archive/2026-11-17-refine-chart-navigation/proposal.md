# Proposal

## Why

The monthly cost/distance navigation added in `restore-chart-history` has three problems. On phones, the date range, navigation, and mode toggle wrap onto three header rows, which makes the card about 500px tall and stretches every carousel card to that height. On desktop, the oversized arrow buttons and dark date range dominate the header, and the date range repeats what the category axis already shows. The 12-month jumps only reach fixed windows counted back from the latest month, so a user in November cannot view January–December.

## What Changes

- Monthly navigation moves by one month (‹ ›) or by twelve months (« »), so any 12-month window, such as a calendar year, can be reached.
- Remove the visible date range. The category axis already labels every visible month.
- Show the navigation as a compact control that matches the monthly/yearly toggle.
- Hide monthly navigation in the compact (below 960px) layout. Compact monthly charts show the latest 12 months, or the window already selected in the session.
- Update the English/German accessible names: "Previous/Next month" and "Previous/Next year".

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `ui-charts`: Monthly navigation steps by one or twelve months, shows no visible date range, and is desktop-only.

## Impact

- Frontend only: `ui/src/lib/monthly-window.ts`, `PeriodNavigation.svelte`, the cost/distance chart components, i18n strings, and their tests.
- No API, backend, schema, or dependency changes.
