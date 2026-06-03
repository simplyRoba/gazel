// ── Numeric input guarding ───────────────────────────────
//
// Pure helpers for keeping numeric text inputs (odometer, fuel, price, cost)
// free of non-numeric garbage while typing, independent of the locale's
// decimal separator. Parsing of completed values lives in `format.ts`
// (`parseDecimal`); these helpers only gate intermediate keystrokes.

import { formatDecimalInput, type FuelPriceTotalField } from "$lib/format";

/**
 * Returns `true` when `value` is a valid *partial* numeric entry — i.e. a
 * string the user could still be typing toward a real number.
 *
 * Accepts:
 * - the empty string (field being cleared)
 * - an optional single leading `-` when `allowNegative` is set
 * - digits
 * - at most one decimal separator (`.` or `,`), in any position
 *
 * Rejects letters, whitespace, multiple separators, multiple signs, etc.
 * Grouping separators are intentionally NOT accepted mid-typing; they are
 * handled on blur via normalization.
 */
export function isValidPartialNumber(
  value: string,
  allowNegative: boolean = false,
): boolean {
  if (value === "") return true;
  const pattern = allowNegative ? /^-?\d*([.,]\d*)?$/ : /^\d*([.,]\d*)?$/;
  return pattern.test(value);
}

/**
 * Compute the prospective input value if `insert` were applied to `current`
 * with the given selection range. Used to validate a keystroke before it
 * lands. A `null`/`undefined` selection is treated as a caret at the end.
 */
export function applyInput(
  current: string,
  insert: string,
  selectionStart: number | null | undefined,
  selectionEnd: number | null | undefined,
): string {
  const start = selectionStart ?? current.length;
  const end = selectionEnd ?? current.length;
  return current.slice(0, start) + insert + current.slice(end);
}

/**
 * `beforeinput` handler factory that blocks keystrokes which would make the
 * field hold a non-numeric value. Deletions and non-insert input types are
 * always allowed; the field is re-validated against `isValidPartialNumber`.
 */
export function guardNumericBeforeInput(allowNegative: boolean = false) {
  return (event: InputEvent): void => {
    const target = event.target as HTMLInputElement | null;
    if (!target) return;
    // Let pastes/drops through — they may carry grouping/currency formatting
    // (e.g. "1,234.56" or "€ 50.00") that on-blur normalization cleans up.
    if (
      event.inputType === "insertFromPaste" ||
      event.inputType === "insertFromDrop"
    ) {
      return;
    }
    // Allow deletions and composition/other input types.
    if (event.data == null) return;
    const next = applyInput(
      target.value,
      event.data,
      target.selectionStart,
      target.selectionEnd,
    );
    if (!isValidPartialNumber(next, allowNegative)) {
      event.preventDefault();
    }
  };
}

/**
 * Normalize a numeric input's value on blur into the locale's display form.
 * Returns the value to assign back to the field.
 */
export function normalizeOnBlur(
  value: string,
  locale: string,
  maxDecimals: number = 3,
): string {
  return formatDecimalInput(value, locale, maxDecimals);
}

// ── Fuel / price / total edit ordering ───────────────────

/**
 * Update the most-recently-edited ordering for the fuel/price/total trio.
 * The just-edited field moves to the front; the list stays length 3.
 *
 * The first two entries are the "authoritative" fields (user-provided); the
 * last entry is the one auto-calc may recompute.
 */
export function recordFieldEdit(
  order: FuelPriceTotalField[],
  edited: FuelPriceTotalField,
): FuelPriceTotalField[] {
  return [edited, ...order.filter((f) => f !== edited)].slice(0, 3);
}
