// ── Number parsing helper ────────────────────────────────

/**
 * Resolve the grouping and decimal separators for a locale.
 * Falls back to `,` / `.` if the locale is unknown.
 */
function getSeparators(locale: string): { group: string; decimal: string } {
  try {
    const parts = new Intl.NumberFormat(locale).formatToParts(11111.1);
    const group = parts.find((p) => p.type === "group")?.value ?? ",";
    const decimal = parts.find((p) => p.type === "decimal")?.value ?? ".";
    return { group, decimal };
  } catch {
    return { group: ",", decimal: "." };
  }
}

/**
 * Parse a user-entered number string that may use either `.` or `,` as the
 * decimal separator (and optionally grouping separators).
 *
 * This makes numeric inputs forgiving across locales and mobile keyboards
 * that only expose a comma (e.g. German) — "477,2" parses to 477.2.
 *
 * Disambiguation rules:
 * - If both separators appear, the last-occurring one is the decimal point
 *   and the other is grouping ("1.234,56" → 1234.56, "1,234.56" → 1234.56).
 * - If only one separator appears once and is NOT followed by exactly three
 *   digits, it is treated as the decimal point ("477,2" → 477.2).
 * - If a single separator is followed by exactly three digits (ambiguous,
 *   e.g. "234.567"), the active locale decides whether it is decimal or
 *   grouping.
 * - Multiple occurrences of one separator are always grouping.
 *
 * Returns `NaN` for empty or unparseable input.
 */
export function parseDecimal(
  input: string | number | null | undefined,
  locale: string = "en",
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
    const digitsAfter = s.length - s.lastIndexOf(sep) - 1;
    let isDecimal: boolean;
    if (occurrences > 1) {
      isDecimal = false;
    } else if (digitsAfter === 3) {
      isDecimal = sep === getSeparators(locale).decimal;
    } else {
      isDecimal = true;
    }
    s = isDecimal ? s.replace(sep, ".") : s.split(sep).join("");
  }

  return Number(s);
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
