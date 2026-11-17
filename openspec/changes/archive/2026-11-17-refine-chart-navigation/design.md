# Design

## Decisions

- **Step sizes:** `monthlyWindow().move(direction, step)` takes `step` of `1 | 12` and defaults to `1`. The existing clamping keeps every window inside the source span. The window stays keyed by its calendar start month, so refresh and vehicle-selection behavior is unchanged. A 12-month fast step keeps the same calendar months in view (Jan–Dec → Jan–Dec of the previous year), which makes year-over-year comparison easy. Arbitrary 5- or 10-month jumps would not.
- **Control styling:** the four buttons reuse the shared `segmented` / `segmented-item` styles. The 0.85 header scale moves from the two chart components into a shared `.segmented-sm` rule in `ui/src/lib/styles/segmented.css`, used by the navigation and the mode toggle, so heights and borders match. No new design tokens.
- **No visible date range:** accessible names describe each action. The axis labels identify the visible months.
- **Compact layout:** a CSS media query (`max-width: 959px`) hides the navigation, matching the dashboard's existing carousel breakpoint. This needs no JS viewport detection. Measured at 390px, the content box is 332px wide, and title + controls + toggle need about 383px, so keeping the controls would always add a header row.

## Trade-offs

- Compact-layout hiding is CSS-only. jsdom does not evaluate media queries, so it is checked by responsive screenshots rather than unit tests. Automating it would require a real-browser test runner, which is not worth adding for one rule.
- If a user selects an older window on desktop and then narrows the viewport in the same session, the compact chart keeps showing that window without controls. This is acceptable because selection is session state only and resets on vehicle change or reload.
