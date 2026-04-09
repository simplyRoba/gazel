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

export function formatEfficiency(
  value: number,
  distanceUnit: string,
  volumeUnit: string,
): string {
  const rounded = value.toFixed(1);
  if (distanceUnit === "mi" && volumeUnit === "gal") {
    return `${rounded} mpg`;
  }
  const volLabel = VOLUME_LABELS[volumeUnit] ?? volumeUnit;
  return `${rounded} ${distanceUnit}/${volLabel}`;
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
