// ── Distance ─────────────────────────────────────────────

export function formatDistance(value: number, unit: string): string {
  const rounded = value.toFixed(1);
  return `${rounded} ${unit}`;
}

// ── Volume ───────────────────────────────────────────────

const VOLUME_LABELS: Record<string, string> = {
  l: "L",
  gal: "gal",
};

export function formatVolume(value: number, unit: string): string {
  const rounded = value.toFixed(1);
  const label = VOLUME_LABELS[unit] ?? unit;
  return `${rounded} ${label}`;
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
): string {
  const display = toDisplayEfficiency(value, distanceUnit, volumeUnit);
  const rounded = display.toFixed(1);
  const unit = efficiencyUnitLabel(distanceUnit, volumeUnit);
  return `${rounded} ${unit}`;
}

// ── Currency ─────────────────────────────────────────────

const CURRENCY_SYMBOLS: Record<string, string> = {
  USD: "$",
  EUR: "\u20AC",
};

export function formatCurrency(value: number, currency: string): string {
  const rounded = value.toFixed(2);
  const symbol = CURRENCY_SYMBOLS[currency];
  if (symbol !== undefined) {
    return `${symbol}${rounded}`;
  }
  return `${currency} ${rounded}`;
}
