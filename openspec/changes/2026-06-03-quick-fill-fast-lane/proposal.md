## Why

Logging a fill-up is the single most frequent user interaction in gazel — it is the entire purpose of the app. It happens at the pump: on mobile, one-handed, in a hurry. The current `FillupForm` is a full detail form (date, odometer, fuel, cost, station, notes, two toggles) that optimizes for completeness over speed. Real-world entry suffers:

- Users must type three numbers (fuel amount, cost, and mentally reconcile the per-litre price they see on the pump) with no help.
- All fields are shown at once, including ones almost never changed on the go (date = today, station, notes, full-tank = true).
- Numeric input was fragile (a comma-decimal bug was recently fixed); there is still no input guarding or on-blur normalization, so it does not feel "solid."

This change introduces a **Quick Fill fast lane**: a mobile-first, single-screen entry optimized for speed and trust, while keeping the existing detailed form for editing and full control.

## What Changes

- Add a **locale-aware decimal input behavior** to numeric fields: forgiving parse (already added via `parseDecimal`), keypress guarding (reject characters that cannot form a valid number), and on-blur normalization to the user's locale display. Extract this into a reusable helper/action so both Quick Fill and the detailed form benefit.
- Add a **price/total auto-calc** capability: the three pump values — fuel amount (volume), price per unit, and total cost — are mutually derived. Entering any two computes the third. This removes mental math and prevents inconsistent data.
- Add a new **Quick Fill** entry mode (single-screen fast lane) used as the default for "Add fill-up":
  - Large, thumb-friendly numeric inputs for the receipt numbers (fuel amount, price/unit, total) plus odometer (prefilled from last reading, in the user's chosen total/trip mode).
  - **Live feedback**: as odometer + fuel are entered, show the computed efficiency (`L/100km` or `mpg` per settings) and confirm the total, so the user instantly sees the entry is correct.
  - A single primary **Save** action. Date (defaults to today), station, notes, full-tank toggle, and missed-fill-up handling are collapsed behind a single "More details" expander; most fill-ups never open it.
  - Reuses existing validation rules, the smart missed-fill-up prompt, and the existing fill-up store create action.
- Keep the existing detailed **`FillupForm`** as the **edit** experience (tapping a fill-up card) and as the expanded "More details" surface, so no functionality is lost.
- Add i18n keys (en + de) for the new Quick Fill labels, price/unit field, auto-calc hints, live efficiency preview, and the "More details" expander.

## Capabilities

### New Capabilities

- `quick-fill`: A mobile-first single-screen fill-up entry mode with large numeric inputs, fuel/price/total auto-calc, live efficiency preview, a collapsible "More details" section, and a single primary Save action. Reuses existing validation, smart missed-fill-up detection, and the fill-up store.

### Modified Capabilities

- `fillup-ui`: The "Add fill-up" CTA and dashboard add button SHALL open Quick Fill by default; the existing detailed form SHALL remain the edit experience and the source of the expanded fields. Form numeric inputs SHALL use locale-aware parsing, keypress guarding, and on-blur normalization.
- `unit-formatting`: A reusable locale-aware decimal **parse** helper (`parseDecimal`) and a blur **normalization** helper SHALL be specified alongside the existing format helpers, including disambiguation rules for `.` vs `,` as decimal/grouping separators.

Quick Fill labels still require new i18n keys, but no new i18n *spec requirement* is added — the existing i18n spec already mandates that every `en.json` key has a matching `de.json` entry.

## Impact

- **UI files**:
  - `ui/src/lib/components/QuickFillForm.svelte` (new) — the fast-lane screen.
  - `ui/src/lib/components/FillupModal.svelte` (modified) — host Quick Fill for create, detailed form for edit.
  - `ui/src/lib/components/FillupForm.svelte` (modified) — share numeric input behavior; serve as edit + expanded surface.
  - `ui/src/lib/format.ts` (modified/confirmed) — `parseDecimal` already added; add a `formatDecimalInput` helper for on-blur normalization and an auto-calc helper (`deriveFuelPriceTotal`).
  - `ui/src/lib/i18n/en.json`, `ui/src/lib/i18n/de.json` (new keys).
  - Possibly a small input action/util (`ui/src/lib/numeric-input.ts`) for keypress guarding + blur normalization.
- **Tests**: unit tests for the auto-calc helper, `parseDecimal` (already added) plus normalization, and component tests for Quick Fill (auto-calc behavior, validation, live preview, expander, save). Existing `FillupForm` tests remain.
- **No backend changes**: frontend-only; `CreateFillup` payload shape is unchanged.
- **No new dependencies**: native browser APIs (`Intl`) and Svelte 5 runes only.
- **No breaking changes**: editing and the detailed form are preserved; Quick Fill is additive for the create path.
