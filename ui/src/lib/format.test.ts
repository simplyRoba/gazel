import { describe, expect, it } from "vitest";
import {
  formatCurrency,
  formatDistance,
  formatEfficiency,
  formatVolume,
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
});

describe("formatEfficiency", () => {
  it("formats metric efficiency (km/L)", () => {
    expect(formatEfficiency(15.3, "km", "l")).toBe("15.3 km/L");
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
});
