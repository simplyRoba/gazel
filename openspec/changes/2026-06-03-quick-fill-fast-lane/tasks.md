## 1. Shared numeric input behavior

- [ ] 1.1 Confirm/keep `parseDecimal(value, locale)` in `ui/src/lib/format.ts` (already added) and ensure exported
- [ ] 1.2 Add `formatDecimalInput(value, locale, decimals?)` to `ui/src/lib/format.ts` for on-blur normalization
- [ ] 1.3 Add a numeric input helper/action (e.g. `ui/src/lib/numeric-input.ts`) for keypress guarding (digits, single decimal separator, optional leading sign) and blur normalization
- [ ] 1.4 Unit tests for `formatDecimalInput` and the keypress-guard predicate

## 2. Fuel / price / total auto-calc

- [ ] 2.1 Add pure helper `deriveFuelPriceTotal(fields, lastEdited)` in `ui/src/lib/format.ts` (or sibling util) implementing `total = fuel * price`
- [ ] 2.2 Implement last-edited authority rule (only the least-recently-edited field is recomputed; focused field never overwritten)
- [ ] 2.3 Division guards (no compute when dividing by zero/invalid; never blocks typing)
- [ ] 2.4 Unit tests: compute each of the three from the other two, last-edited rule, division guards, rounding precision

## 3. Quick Fill component

- [ ] 3.1 Create `ui/src/lib/components/QuickFillForm.svelte` — large numeric inputs for fuel, price/unit, total, and odometer; single primary Save
- [ ] 3.2 Prefill odometer from last reading (respecting active total/trip mode); date defaults to today
- [ ] 3.3 Wire auto-calc to the three numeric fields using `deriveFuelPriceTotal` + last-edited tracking
- [ ] 3.4 Live efficiency preview using existing `formatEfficiency`/`toDisplayEfficiency` and the previous full-tank odometer; hidden when insufficient data
- [ ] 3.5 "More details" expander (collapsed by default) revealing date, station, notes, full-tank toggle, missed toggle
- [ ] 3.6 Reuse validation rules (required date/odometer/fuel/total, positivity, odometer-min) on Save
- [ ] 3.7 Reuse smart missed-fill-up prompt
- [ ] 3.8 Build `CreateFillup` payload with `fuel_amount` and `cost` only (price/unit not sent); call existing store create

## 4. Modal wiring

- [ ] 4.1 Update `ui/src/lib/components/FillupModal.svelte` to render Quick Fill on create and the detailed `FillupForm` on edit
- [ ] 4.2 Keep the total/trip mode toggle in the modal header applying to both surfaces
- [ ] 4.3 Ensure dashboard add button and global CTA open Quick Fill for create

## 5. Detailed form alignment

- [ ] 5.1 Route the detailed `FillupForm` numeric inputs through the shared parse/guard/normalize behavior
- [ ] 5.2 Reuse detailed fields inside the Quick Fill "More details" section where practical (avoid duplication)

## 6. i18n

- [ ] 6.1 Add Quick Fill keys to `ui/src/lib/i18n/en.json` (title, price/unit label, live preview label, auto-calc hint, "More details")
- [ ] 6.2 Add matching German translations to `ui/src/lib/i18n/de.json`

## 7. Tests

- [ ] 7.1 Component tests for Quick Fill: auto-calc behavior, validation on save, live preview show/hide, expander, successful create
- [ ] 7.2 Ensure existing `FillupForm` and dashboard tests still pass
- [ ] 7.3 Run full pre-review gate (format:check, lint, check, vitest, cargo test)

## 8. Spec & docs

- [ ] 8.1 Verify implementation matches the spec deltas (quick-fill, fillup-ui, unit-formatting)
- [ ] 8.2 Update any developer notes if the create/edit surfaces are referenced elsewhere
