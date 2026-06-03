import { describe, expect, it } from "vitest";
import {
  deriveFuelPriceTotal,
  formatCurrency,
  formatDecimalInput,
  formatDistance,
  formatEfficiency,
  formatVolume,
  parseDecimal,
  toDisplayEfficiency,
  efficiencyUnitLabel,
  isLper100km,
} from "./format";

describe("formatDecimalInput", () => {
  it("normalizes a comma decimal to English display", () => {
    expect(formatDecimalInput("477,2", "en")).toBe("477.2");
  });

  it("normalizes a dot decimal to German display", () => {
    expect(formatDecimalInput("477.2", "de")).toBe("477,2");
  });

  it("strips grouping separators", () => {
    expect(formatDecimalInput("1.234,56", "de")).toBe("1234,56");
  });

  it("does not force trailing zeros", () => {
    expect(formatDecimalInput("45", "en")).toBe("45");
    expect(formatDecimalInput("45.50", "en")).toBe("45.5");
  });

  it("caps fraction digits at maxDecimals", () => {
    expect(formatDecimalInput("1.23456", "en", 3)).toBe("1.235");
  });

  it("returns empty string for blank/nullish input", () => {
    expect(formatDecimalInput("", "en")).toBe("");
    expect(formatDecimalInput("   ", "en")).toBe("");
    expect(formatDecimalInput(null, "en")).toBe("");
    expect(formatDecimalInput(undefined, "en")).toBe("");
  });

  it("returns the original string when unparseable", () => {
    expect(formatDecimalInput("abc", "en")).toBe("abc");
  });
});

describe("deriveFuelPriceTotal", () => {
  it("computes total from fuel and price", () => {
    expect(
      deriveFuelPriceTotal({ fuel: 40, price: 1.5 }, ["fuel", "price"]),
    ).toEqual({ field: "total", value: 60 });
  });

  it("computes price from fuel and total", () => {
    expect(
      deriveFuelPriceTotal({ fuel: 40, total: 60 }, ["fuel", "total"]),
    ).toEqual({ field: "price", value: 1.5 });
  });

  it("computes fuel from price and total", () => {
    expect(
      deriveFuelPriceTotal({ price: 1.5, total: 60 }, ["price", "total"]),
    ).toEqual({ field: "fuel", value: 40 });
  });

  it("guards against dividing by zero fuel", () => {
    expect(
      deriveFuelPriceTotal({ fuel: 0, total: 60 }, ["fuel", "total"]),
    ).toBeNull();
  });

  it("guards against dividing by zero price", () => {
    expect(
      deriveFuelPriceTotal({ price: 0, total: 60 }, ["price", "total"]),
    ).toBeNull();
  });

  it("returns null when an authoritative value is missing or invalid", () => {
    expect(deriveFuelPriceTotal({ fuel: 40 }, ["fuel", "price"])).toBeNull();
    expect(
      deriveFuelPriceTotal({ fuel: 40, price: NaN }, ["fuel", "price"]),
    ).toBeNull();
  });
});

describe("parseDecimal", () => {
  it("parses a plain integer", () => {
    expect(parseDecimal("477")).toBe(477);
  });

  it("parses a dot decimal", () => {
    expect(parseDecimal("477.2")).toBe(477.2);
  });

  it("parses a comma decimal (the reported bug)", () => {
    expect(parseDecimal("477,2")).toBe(477.2);
  });

  it("parses comma decimal with multiple fraction digits", () => {
    expect(parseDecimal("1234,56")).toBe(1234.56);
  });

  it("handles German grouping + comma decimal", () => {
    expect(parseDecimal("1.234,56", "de")).toBe(1234.56);
  });

  it("handles English grouping + dot decimal", () => {
    expect(parseDecimal("1,234.56", "en")).toBe(1234.56);
  });

  it("treats a single separator with three trailing digits as a decimal (fuel price)", () => {
    // The key fuel-price case: "1,919" is €1.919/L, never 1919.
    expect(parseDecimal("1,919")).toBe(1.919);
    expect(parseDecimal("1.919")).toBe(1.919);
    expect(parseDecimal("234.567")).toBe(234.567);
    expect(parseDecimal("234,567")).toBe(234.567);
  });

  it("treats repeated grouping separators as grouping", () => {
    expect(parseDecimal("1,234,567", "en")).toBe(1234567);
    expect(parseDecimal("1.234.567", "de")).toBe(1234567);
  });

  it("passes numbers through unchanged", () => {
    expect(parseDecimal(42.5)).toBe(42.5);
  });

  it("strips stray symbols and whitespace", () => {
    expect(parseDecimal(" 63,00 € ")).toBe(63);
    expect(parseDecimal("45,2 L")).toBe(45.2);
  });

  it("returns NaN for empty or junk input", () => {
    expect(parseDecimal("")).toBeNaN();
    expect(parseDecimal("   ")).toBeNaN();
    expect(parseDecimal(null)).toBeNaN();
    expect(parseDecimal(undefined)).toBeNaN();
    expect(parseDecimal("abc")).toBeNaN();
  });
});

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
