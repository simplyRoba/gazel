import { describe, expect, it } from "vitest";
import { deriveFuelPriceTotal, type FuelPriceTotalField } from "./format";
import {
  applyInput,
  isValidPartialNumber,
  normalizeOnBlur,
  recordFieldEdit,
} from "./numeric-input";

describe("isValidPartialNumber", () => {
  it("accepts the empty string", () => {
    expect(isValidPartialNumber("")).toBe(true);
  });

  it("accepts plain digits", () => {
    expect(isValidPartialNumber("477")).toBe(true);
  });

  it("accepts a dot decimal in progress", () => {
    expect(isValidPartialNumber("477.")).toBe(true);
    expect(isValidPartialNumber("477.2")).toBe(true);
  });

  it("accepts a comma decimal in progress", () => {
    expect(isValidPartialNumber("477,")).toBe(true);
    expect(isValidPartialNumber("477,2")).toBe(true);
  });

  it("accepts a leading separator", () => {
    expect(isValidPartialNumber(".5")).toBe(true);
    expect(isValidPartialNumber(",5")).toBe(true);
  });

  it("rejects letters and symbols", () => {
    expect(isValidPartialNumber("47a")).toBe(false);
    expect(isValidPartialNumber("4 7")).toBe(false);
    expect(isValidPartialNumber("$4")).toBe(false);
  });

  it("rejects multiple separators", () => {
    expect(isValidPartialNumber("4.7.2")).toBe(false);
    expect(isValidPartialNumber("4,7,2")).toBe(false);
    expect(isValidPartialNumber("4.7,2")).toBe(false);
  });

  it("rejects a sign by default but accepts it when allowed", () => {
    expect(isValidPartialNumber("-5")).toBe(false);
    expect(isValidPartialNumber("-5", true)).toBe(true);
    expect(isValidPartialNumber("5-", true)).toBe(false);
  });
});

describe("applyInput", () => {
  it("appends at the end when selection is null", () => {
    expect(applyInput("47", "2", null, null)).toBe("472");
  });

  it("inserts at the caret position", () => {
    expect(applyInput("472", ".", 2, 2)).toBe("47.2");
  });

  it("replaces the selected range", () => {
    expect(applyInput("4772", "0", 1, 3)).toBe("402");
  });
});

describe("normalizeOnBlur", () => {
  it("normalizes a comma decimal to English display", () => {
    expect(normalizeOnBlur("477,2", "en")).toBe("477.2");
  });

  it("normalizes a dot decimal to German display", () => {
    expect(normalizeOnBlur("477.2", "de")).toBe("477,2");
  });

  it("returns empty string for blank input", () => {
    expect(normalizeOnBlur("", "en")).toBe("");
  });
});

describe("recordFieldEdit", () => {
  it("moves the edited field to the front", () => {
    expect(recordFieldEdit(["fuel", "total", "price"], "price")).toEqual([
      "price",
      "fuel",
      "total",
    ]);
  });

  it("keeps the list at length three with no duplicates", () => {
    const next = recordFieldEdit(["fuel", "total", "price"], "fuel");
    expect(next).toEqual(["fuel", "total", "price"]);
    expect(next).toHaveLength(3);
  });
});

describe("auto-calc flow (recordFieldEdit + deriveFuelPriceTotal)", () => {
  // Simulates the component: track edit order, derive the third (last) field.
  function calc(
    order: FuelPriceTotalField[],
    edited: FuelPriceTotalField,
    fields: { fuel?: number; price?: number; total?: number },
  ) {
    const next = recordFieldEdit(order, edited);
    const authoritative = next.slice(0, 2) as [
      FuelPriceTotalField,
      FuelPriceTotalField,
    ];
    return {
      order: next,
      result: deriveFuelPriceTotal(fields, authoritative),
    };
  }

  it("computes total after editing fuel then price", () => {
    let order: FuelPriceTotalField[] = ["fuel", "total", "price"];
    ({ order } = calc(order, "fuel", { fuel: 40 }));
    const { result } = calc(order, "price", { fuel: 40, price: 1.5 });
    expect(result).toEqual({ field: "total", value: 60 });
  });

  it("recomputes the oldest field when the computed field is edited", () => {
    // Start: fuel + price authoritative, total derived.
    const order: FuelPriceTotalField[] = ["price", "fuel", "total"];
    // User now edits total -> total + price authoritative, fuel recomputed.
    const { result } = calc(order, "total", { price: 1.5, total: 90 });
    expect(result).toEqual({ field: "fuel", value: 60 });
  });

  it("never recomputes the field just edited", () => {
    const order: FuelPriceTotalField[] = ["fuel", "price", "total"];
    const { result } = calc(order, "total", {
      fuel: 40,
      price: 1.5,
      total: 99,
    });
    expect(result?.field).not.toBe("total");
  });
});
