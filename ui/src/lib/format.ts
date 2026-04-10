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
