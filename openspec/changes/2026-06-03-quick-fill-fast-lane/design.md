## Context

The fill-up create/edit experience lives in three UI files:

- `FillupModal.svelte` — `<dialog>` wrapper; holds the total/trip mode toggle, the title, and renders `FillupForm`. It calls a store `createFillup`/`updateFillup` via the dashboard's `onsave` handler.
- `FillupForm.svelte` — the detailed form (date, odometer with total/trip modes, fuel, cost, station, notes, full-tank + missed toggles), client-side validation, and the smart missed-fill-up prompt.
- `format.ts` — locale-aware display formatters and the recently added `parseDecimal` parser.

Numeric fields are `type="text"` + `inputmode="decimal"`, bound to strings, parsed on submit. A comma-decimal truncation bug was already fixed by routing parsing through `parseDecimal(value, locale)`.

The goal is a faster create path without losing the detailed form (still needed for edit and for less-common fields).

## Goals / Non-Goals

**Goals:**
- A single-screen, mobile-first **Quick Fill** create experience with large touch targets and minimal taps.
- **Auto-calc** between fuel amount, price-per-unit, and total so users enter only what they read off the pump.
- **Live efficiency preview** so entry feels rewarding and typos are caught immediately.
- Bulletproof numeric input: forgiving parse, keypress guarding, on-blur normalization — shared by Quick Fill and the detailed form.
- Reuse existing validation, smart missed-fill-up detection, and the fill-up store. No backend changes.

**Non-Goals:**
- Changing the `CreateFillup`/`UpdateFillup` API contract or backend.
- Replacing the detailed `FillupForm` (it remains for edit and the expanded section).
- A multi-step click-through wizard (rejected — see Decision 1).
- Receipt OCR / camera capture, voice entry, or persisted "last station" autocomplete (possible future work).
- Currency/unit conversion beyond display.

## Decisions

### 1. Single-screen fast lane, not a multi-step wizard

**Decision**: Quick Fill is one screen with a collapsible "More details" section, not a one-field-per-step wizard.

**Rationale**: For a 3–4 value entry, a step-per-field wizard adds screen transitions and "Next" taps and removes the at-a-glance overview that lets users confirm the whole entry before saving. The speed wins come from big targets, auto-advance, fewest required values, and instant feedback — all achievable on one screen. The single screen is faster than both the current form and a wizard.

**Alternatives**: A 2-step wizard (numbers → optional details) was considered; rejected for the extra transition with no real speed benefit, but the collapsible section preserves its "hide the rare fields" advantage.

### 2. Auto-calc as a pure helper with an explicit "last edited" rule

**Decision**: Add a pure helper `deriveFuelPriceTotal(fields, lastEdited)` in `format.ts` (or a sibling util). The three values relate by `total = fuel * pricePerUnit`. The two most-recently-edited fields are authoritative; the third is computed. Editing the computed field makes it authoritative and recomputes the *oldest* of the other two.

**Rationale**: Keeping derivation pure and explicit (driven by which field the user last touched) avoids ambiguous circular updates and is fully unit-testable. The component only tracks "which two are user-provided."

**Details:**
- If exactly two of {fuel, price, total} are present/valid, compute the third.
- Division guards: if `fuel <= 0`, do not compute price from total; leave price empty and surface a validation error on submit only.
- Rounding for display only: price to a sensible precision (e.g. 3 decimals), total to 2, fuel to 2. The authoritative stored values are `fuel_amount` and `cost`; `pricePerUnit` is a UI convenience and is NOT sent to the API.
- The payload still sends `fuel_amount` and `cost` exactly as today.

### 3. Shared numeric input behavior (parse + guard + normalize)

**Decision**: Centralize numeric input behavior:
- `parseDecimal(value, locale)` — already added; forgiving parse of `.`/`,`.
- `formatDecimalInput(value, locale, decimals?)` — normalize a parsed value back to the locale's display string for on-blur.
- A small Svelte action or shared handler for **keypress guarding**: allow digits, one decimal separator (locale's or `.`/`,`), and a leading sign where relevant; reject everything else so the field can never hold garbage.

**Rationale**: Both Quick Fill and the detailed form need identical, trustworthy input. A single source avoids drift and is testable in isolation.

### 4. Live efficiency preview reuses existing format/stat logic

**Decision**: Compute the preview from the entered odometer (resolved via the existing total/trip logic) and fuel amount, using the existing `formatEfficiency` / `toDisplayEfficiency` helpers and the previous fill-up's odometer for the segment distance. Show it only when enough data is present (a previous full-tank reading exists and distance > 0).

**Rationale**: Reuses tested formatting and the same `L/100km` vs `mpg` rules as the rest of the app; no new efficiency math.

### 5. Routing of create vs edit

**Decision**: `FillupModal` opens **Quick Fill** when creating (no `initial`) and the detailed **FillupForm** when editing (`initial` present). "More details" inside Quick Fill expands to the same detailed fields (date, station, notes, toggles) rather than navigating away. The total/trip mode toggle stays in the modal header and applies to both.

**Rationale**: Preserves all edit functionality and the recently fixed total/trip toggle behavior, while making the common create path fast. Avoids duplicating the detailed fields by reusing `FillupForm`'s field building where practical.

### 6. Validation and smart prompt reuse

**Decision**: Quick Fill reuses the same validation rules (required date/odometer/fuel/cost, positivity, odometer-min constraint) and the smart missed-fill-up prompt already implemented for `FillupForm`. Validation runs on Save; auto-calc never blocks typing.

**Rationale**: Consistent behavior and a single set of rules to maintain; the missed-fill-up heuristic is valuable exactly in the on-the-go create path.

## Risks / Trade-offs

- **Two create surfaces sharing fields** risks duplication/drift. Mitigation: share field building and the numeric input behavior; keep `FillupForm` as the canonical detailed surface used inside the expander.
- **Auto-calc surprise** (user edits a value and an unexpected field changes). Mitigation: explicit last-edited rule, only ever computes the single non-authoritative field, and never overwrites the field currently focused.
- **Per-unit precision** could introduce rounding drift between price and total. Mitigation: store only `fuel_amount` and `cost`; treat price as derived/display; round consistently.

## Migration / Rollout

- Additive: editing is unchanged; create defaults to Quick Fill. No data migration. No API change.
- Behind no flag initially; if needed, the "Add fill-up" entry can fall back to the detailed form trivially since both share the store call.
