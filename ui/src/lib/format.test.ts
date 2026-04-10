import { describe, expect, it } from "vitest";
import {
  formatCurrency,
  formatDistance,
  formatEfficiency,
  formatVolume,
  toDisplayEfficiency,
  efficiencyUnitLabel,
  isLper100km,
} from "./format";

describe("formatDistance", () => {
  it("formats kilometers", () => {
    expect(formatDistance(142.5, "km")).toBe("142.5 km");
  });

  it("formats miles", () => {
    expect(formatDistance(88.6, "mi")).toBe("88.6 mi");
  });

  it("rounds to one decimal place", () => {
    expect(formatDistance(100.456, "km")).toBe("100.5 km");
  });

  it("uses locale-aware formatting for German", () => {
    expect(formatDistance(1142.5, "km", "de")).toBe("1.142,5 km");
  });

  it("defaults to English locale", () => {
    expect(formatDistance(1142.5, "km")).toBe("1,142.5 km");
  });
});

describe("formatVolume", () => {
  it("formats liters with uppercase L", () => {
    expect(formatVolume(45.2, "l")).toBe("45.2 L");
  });

  it("formats gallons", () => {
    expect(formatVolume(12.0, "gal")).toBe("12.0 gal");
  });

  it("rounds to one decimal place", () => {
    expect(formatVolume(33.789, "l")).toBe("33.8 L");
  });

  it("uses German locale formatting", () => {
    expect(formatVolume(45.2, "l", "de")).toBe("45,2 L");
  });
});

describe("isLper100km", () => {
  it("returns true for km + l", () => {
    expect(isLper100km("km", "l")).toBe(true);
  });

  it("returns false for imperial", () => {
    expect(isLper100km("mi", "gal")).toBe(false);
  });

  it("returns false for mixed units", () => {
    expect(isLper100km("km", "gal")).toBe(false);
  });
});

describe("toDisplayEfficiency", () => {
  it("converts km/L to L/100km for metric", () => {
    // 10 km/L = 10 L/100km
    expect(toDisplayEfficiency(10, "km", "l")).toBeCloseTo(10.0);
    // 12.5 km/L = 8 L/100km
    expect(toDisplayEfficiency(12.5, "km", "l")).toBeCloseTo(8.0);
    // 7.7 km/L ≈ 12.99 L/100km
    expect(toDisplayEfficiency(7.7, "km", "l")).toBeCloseTo(12.987, 2);
  });

  it("returns value as-is for imperial", () => {
    expect(toDisplayEfficiency(32.1, "mi", "gal")).toBe(32.1);
  });

  it("handles zero gracefully", () => {
    expect(toDisplayEfficiency(0, "km", "l")).toBe(0);
  });
});

describe("efficiencyUnitLabel", () => {
  it("returns L/100 km for metric", () => {
    expect(efficiencyUnitLabel("km", "l")).toBe("L/100 km");
  });

  it("returns mpg for imperial", () => {
    expect(efficiencyUnitLabel("mi", "gal")).toBe("mpg");
  });

  it("returns composite label for mixed units", () => {
    expect(efficiencyUnitLabel("km", "gal")).toBe("km/gal");
  });
});

describe("formatEfficiency", () => {
  it("formats metric efficiency as L/100km", () => {
    // 12.5 km/L = 8.0 L/100km
    expect(formatEfficiency(12.5, "km", "l")).toBe("8.0 L/100 km");
  });

  it("formats imperial efficiency as mpg", () => {
    expect(formatEfficiency(32.1, "mi", "gal")).toBe("32.1 mpg");
  });

  it("formats mixed units", () => {
    expect(formatEfficiency(20.0, "km", "gal")).toBe("20.0 km/gal");
  });

  it("rounds to one decimal place", () => {
    expect(formatEfficiency(28.456, "mi", "gal")).toBe("28.5 mpg");
  });

  it("uses German locale formatting", () => {
    expect(formatEfficiency(12.5, "km", "l", "de")).toBe("8,0 L/100 km");
  });
});

describe("formatCurrency", () => {
  it("formats USD", () => {
    expect(formatCurrency(42.5, "USD")).toBe("$42.50");
  });

  it("formats EUR", () => {
    expect(formatCurrency(42.5, "EUR")).toBe("\u20AC42.50");
  });

  it("rounds to two decimal places", () => {
    expect(formatCurrency(10.999, "USD")).toBe("$11.00");
  });

  it("falls back to code prefix for unknown currency", () => {
    expect(formatCurrency(42.5, "XYZ")).toBe("XYZ 42.50");
  });

  it("uses German locale formatting for EUR", () => {
    expect(formatCurrency(1042.5, "EUR", "de")).toBe("\u20AC1.042,50");
  });

  it("uses German locale formatting for USD", () => {
    expect(formatCurrency(1042.5, "USD", "de")).toBe("$1.042,50");
  });
});
