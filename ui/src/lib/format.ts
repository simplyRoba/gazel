// ── Number parsing helper ────────────────────────────────

/**
 * Parse a user-entered number string that may use either `.` or `,` as the
 * decimal separator (and optionally grouping separators).
 *
 * This makes numeric inputs forgiving across locales and mobile keyboards
 * that only expose a comma (e.g. German) — "477,2" parses to 477.2.
 *
 * Disambiguation rules (tuned for fill-up entry, where people type decimals,
 * not thousand separators):
 * - If both separators appear, the last-occurring one is the decimal point
 *   and the other is grouping ("1.234,56" → 1234.56, "1,234.56" → 1234.56).
 * - A single separator (of either kind) is ALWAYS the decimal point, even with
 *   three trailing digits ("1,919" → 1.919, "477,2" → 477.2).
 * - Repeated occurrences of one separator are grouping ("1.234.567" → 1234567).
 *
 * The `locale` parameter is accepted for API symmetry with the formatters but
 * is not needed for parsing under these rules.
 *
 * Returns `NaN` for empty or unparseable input.
 */
export function parseDecimal(
  input: string | number | null | undefined,
  _locale: string = "en",
): number {
  if (typeof input === "number") return input;
  if (input === null || input === undefined) return NaN;

  // Drop whitespace (incl. non-breaking/narrow no-break spaces) and any
  // stray non-numeric characters (currency symbols, unit labels, etc.).
  let s = String(input)
    .trim()
    .replace(/[^0-9.,+-]/g, "");
  if (s === "") return NaN;

  const hasComma = s.includes(",");
  const hasDot = s.includes(".");

  if (hasComma && hasDot) {
    const decimalSep = s.lastIndexOf(",") > s.lastIndexOf(".") ? "," : ".";
    const groupSep = decimalSep === "," ? "." : ",";
    s = s.split(groupSep).join("").replace(decimalSep, ".");
  } else if (hasComma || hasDot) {
    const sep = hasComma ? "," : ".";
    const occurrences = s.split(sep).length - 1;
    // A single separator is always the decimal point — users entering fill-up
    // data type decimals, not thousand separators ("1,919" => 1.919). Only
    // repeated separators (e.g. "1.234.567") are treated as grouping.
    s = occurrences > 1 ? s.split(sep).join("") : s.replace(sep, ".");
  }

  return Number(s);
}

/**
 * Normalize a user-entered numeric string into the locale's display form.
 *
 * Intended for on-blur normalization of numeric inputs: parses leniently via
 * `parseDecimal` and re-renders with the locale's decimal separator. Trailing
 * zeros are not forced — up to `maxDecimals` fraction digits are kept.
 *
 * Returns the original input unchanged when it cannot be parsed (so the user
 * keeps the chance to fix it) and an empty string for blank input.
 */
export function formatDecimalInput(
  input: string | number | null | undefined,
  locale: string = "en",
  maxDecimals: number = 3,
): string {
  if (input === null || input === undefined) return "";
  if (typeof input === "string" && input.trim() === "") return "";

  const value = parseDecimal(input, locale);
  if (Number.isNaN(value)) return typeof input === "string" ? input : "";

  return new Intl.NumberFormat(locale, {
    minimumFractionDigits: 0,
    maximumFractionDigits: maxDecimals,
    useGrouping: false,
  }).format(value);
}

// ── Number formatting helper ─────────────────────────────

function formatNumber(
  value: number,
  decimals: number,
  locale: string = "en",
): string {
  return new Intl.NumberFormat(locale, {
    minimumFractionDigits: decimals,
    maximumFractionDigits: decimals,
  }).format(value);
}

// ── Distance ─────────────────────────────────────────────

export function formatDistance(
  value: number,
  unit: string,
  locale: string = "en",
): string {
  return `${formatNumber(value, 1, locale)} ${unit}`;
}

// ── Volume ───────────────────────────────────────────────

const VOLUME_LABELS: Record<string, string> = {
  l: "L",
  gal: "gal",
};

export function formatVolume(
  value: number,
  unit: string,
  locale: string = "en",
): string {
  const label = VOLUME_LABELS[unit] ?? unit;
  return `${formatNumber(value, 1, locale)} ${label}`;
}

// ── Efficiency ───────────────────────────────────────────

/**
 * Returns true when the metric convention L/100km should be used.
 * This applies when the user has km + liters (standard European metric).
 */
export function isLper100km(distanceUnit: string, volumeUnit: string): boolean {
  return distanceUnit === "km" && volumeUnit === "l";
}

/**
 * Convert a distance/volume efficiency value to display value.
 * For metric (km + l): converts km/L → L/100km.
 * For others: returns the value as-is.
 */
export function toDisplayEfficiency(
  value: number,
  distanceUnit: string,
  volumeUnit: string,
): number {
  if (isLper100km(distanceUnit, volumeUnit) && value > 0) {
    return 100 / value;
  }
  return value;
}

/**
 * Returns the efficiency unit label for display.
 */
export function efficiencyUnitLabel(
  distanceUnit: string,
  volumeUnit: string,
): string {
  if (distanceUnit === "mi" && volumeUnit === "gal") return "mpg";
  if (isLper100km(distanceUnit, volumeUnit)) return "L/100 km";
  const volLabel = VOLUME_LABELS[volumeUnit] ?? volumeUnit;
  return `${distanceUnit}/${volLabel}`;
}

export function formatEfficiency(
  value: number,
  distanceUnit: string,
  volumeUnit: string,
  locale: string = "en",
): string {
  const display = toDisplayEfficiency(value, distanceUnit, volumeUnit);
  const unit = efficiencyUnitLabel(distanceUnit, volumeUnit);
  return `${formatNumber(display, 1, locale)} ${unit}`;
}

// ── Currency ─────────────────────────────────────────────

const CURRENCY_SYMBOLS: Record<string, string> = {
  USD: "$",
  EUR: "\u20AC",
};

export function formatCurrency(
  value: number,
  currency: string,
  locale: string = "en",
): string {
  const rounded = formatNumber(value, 2, locale);
  const symbol = CURRENCY_SYMBOLS[currency];
  if (symbol !== undefined) {
    return `${symbol}${rounded}`;
  }
  return `${currency} ${rounded}`;
}

// ── Fuel / price / total derivation ──────────────────────

/** One of the three mutually-derivable fill-up money fields. */
export type FuelPriceTotalField = "fuel" | "price" | "total";

export interface FuelPriceTotal {
  /** Fuel amount (volume). */
  fuel: number;
  /** Price per unit of fuel. */
  price: number;
  /** Total cost (`fuel * price`). */
  total: number;
}

/**
 * Derive the single missing value among fuel amount, price per unit, and total
 * cost using `total = fuel * price`.
 *
 * `authoritative` lists the two fields the user provided (most recently
 * edited). The remaining field is computed from them. Only that one field is
 * returned; the caller decides how to apply it.
 *
 * Returns `null` when the value cannot be derived — missing/invalid inputs, or
 * a division by zero (e.g. computing price/fuel with a zero divisor). Callers
 * SHOULD leave the field untouched (and never block typing) in that case.
 */
export function deriveFuelPriceTotal(
  fields: Partial<FuelPriceTotal>,
  authoritative: [FuelPriceTotalField, FuelPriceTotalField],
): { field: FuelPriceTotalField; value: number } | null {
  const all: FuelPriceTotalField[] = ["fuel", "price", "total"];
  const target = all.find((f) => !authoritative.includes(f));
  if (!target) return null;

  const valid = (n: number | undefined): n is number =>
    typeof n === "number" && Number.isFinite(n);

  const { fuel, price, total } = fields;

  if (target === "total") {
    if (!valid(fuel) || !valid(price)) return null;
    return { field: "total", value: fuel * price };
  }
  if (target === "price") {
    if (!valid(fuel) || !valid(total) || fuel === 0) return null;
    return { field: "price", value: total / fuel };
  }
  // target === "fuel"
  if (!valid(price) || !valid(total) || price === 0) return null;
  return { field: "fuel", value: total / price };
}
