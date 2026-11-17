# Tasks

## 1. Navigation behavior

- [x] 1.1 Add a `1 | 12` step parameter (default 1) to `monthlyWindow().move()` in `ui/src/lib/monthly-window.ts`; update `monthly-window.test.ts` to cover single steps, 12-month steps, clamping at both ends, and reaching a January–December window from a November-ending history.

## 2. Navigation presentation

- [x] 2.1 Rework `PeriodNavigation.svelte` into a compact segmented « ‹ › » control without a visible date range, hidden below 960px; drop the unused `locale` prop from the cost and distance charts.
- [x] 2.2 Update English/German accessible names (`charts.previousPeriod`/`charts.nextPeriod` → month; add `charts.previousYear`/`charts.nextYear`).
- [x] 2.3 Update chart and dashboard tests that relied on 12-month steps or the visible date range.

## 3. Verification

- [x] 3.1 Responsive visual check at 1400px, 390px, and 360px: desktop header on one row with matching control heights; compact header on one row with no navigation.
- [x] 3.2 Validate with `openspec validate refine-chart-navigation --strict`.
- [x] 3.3 Run `cargo fmt -- --check`, `cargo clippy -- -D warnings`, `npm run format:check --prefix ui`, `npm run lint --prefix ui`, `npm run check --prefix ui`, and `cargo test` before archiving.
