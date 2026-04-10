import { describe, it, expect } from "vitest";
import {
  toEfficiencyData,
  toMonthlyCostData,
  toYearlyCostData,
  toMonthlyDistanceData,
  toFuelPriceData,
  toSparklineData,
} from "./charts";
import type { SegmentHistory } from "$lib/api";

function makeSegment(overrides: Partial<SegmentHistory> = {}): SegmentHistory {
  return {
    start_date: "2025-01-01",
    end_date: "2025-01-15",
    start_odometer: 1000,
    end_odometer: 1500,
    distance: 500,
    fuel: 40,
    cost: 60,
    efficiency: 12.5,
    cost_per_distance: 0.12,
    is_valid: true,
    distance_unit: "km",
    volume_unit: "l",
    currency: "USD",
    ...overrides,
  };
}

// ── toEfficiencyData ────────────────────────────────────

describe("toEfficiencyData", () => {
  it("returns empty array for empty input", () => {
    expect(toEfficiencyData([])).toEqual([]);
  });

  it("returns a single point for one valid segment", () => {
    const result = toEfficiencyData([makeSegment()]);
    expect(result).toHaveLength(1);
    expect(result[0].value).toBe(12.5);
    expect(result[0].date).toBeInstanceOf(Date);
  });

  it("filters out invalid segments", () => {
    const segments = [
      makeSegment({ end_date: "2025-01-15", efficiency: 12.5, is_valid: true }),
      makeSegment({
        end_date: "2025-02-15",
        efficiency: 10.0,
        is_valid: false,
      }),
      makeSegment({ end_date: "2025-03-15", efficiency: 14.0, is_valid: true }),
    ];
    const result = toEfficiencyData(segments);
    expect(result).toHaveLength(2);
    expect(result[0].value).toBe(12.5);
    expect(result[1].value).toBe(14.0);
  });

  it("parses end_date into Date objects correctly", () => {
    const result = toEfficiencyData([makeSegment({ end_date: "2025-06-20" })]);
    expect(result[0].date.getFullYear()).toBe(2025);
    expect(result[0].date.getMonth()).toBe(5); // June = 5
    expect(result[0].date.getDate()).toBe(20);
  });
});

// ── toMonthlyCostData ───────────────────────────────────

describe("toMonthlyCostData", () => {
  it("returns empty array for empty input", () => {
    expect(toMonthlyCostData([])).toEqual([]);
  });

  it("aggregates costs within a single month", () => {
    const segments = [
      makeSegment({ end_date: "2025-03-05", cost: 30 }),
      makeSegment({ end_date: "2025-03-20", cost: 45 }),
    ];
    const result = toMonthlyCostData(segments);
    expect(result).toHaveLength(1);
    expect(result[0].month).toBe("2025-03");
    expect(result[0].value).toBe(75);
  });

  it("returns separate entries for different months", () => {
    const segments = [
      makeSegment({ end_date: "2025-01-10", cost: 50 }),
      makeSegment({ end_date: "2025-03-10", cost: 60 }),
      makeSegment({ end_date: "2025-02-10", cost: 40 }),
    ];
    const result = toMonthlyCostData(segments);
    expect(result).toHaveLength(3);
    // Sorted chronologically
    expect(result[0].month).toBe("2025-01");
    expect(result[1].month).toBe("2025-02");
    expect(result[2].month).toBe("2025-03");
  });

  it("fills gaps between months with zero values", () => {
    const segments = [
      makeSegment({ end_date: "2025-01-10", cost: 50 }),
      makeSegment({ end_date: "2025-04-10", cost: 60 }),
    ];
    const result = toMonthlyCostData(segments);
    expect(result).toHaveLength(4);
    expect(result[0]).toEqual({ month: "2025-01", value: 50 });
    expect(result[1]).toEqual({ month: "2025-02", value: 0 });
    expect(result[2]).toEqual({ month: "2025-03", value: 0 });
    expect(result[3]).toEqual({ month: "2025-04", value: 60 });
  });

  it("fills gaps across year boundaries", () => {
    const segments = [
      makeSegment({ end_date: "2024-11-01", cost: 10 }),
      makeSegment({ end_date: "2025-02-01", cost: 20 }),
    ];
    const result = toMonthlyCostData(segments);
    expect(result).toHaveLength(4);
    expect(result[0].month).toBe("2024-11");
    expect(result[1]).toEqual({ month: "2024-12", value: 0 });
    expect(result[2]).toEqual({ month: "2025-01", value: 0 });
    expect(result[3].month).toBe("2025-02");
  });

  it("sorts months chronologically regardless of input order", () => {
    const segments = [
      makeSegment({ end_date: "2025-12-01", cost: 10 }),
      makeSegment({ end_date: "2025-01-01", cost: 20 }),
    ];
    const result = toMonthlyCostData(segments);
    expect(result[0].month).toBe("2025-01");
    expect(result[result.length - 1].month).toBe("2025-12");
    // All 12 months should be present
    expect(result).toHaveLength(12);
  });
});

// ── toYearlyCostData ────────────────────────────────────

describe("toYearlyCostData", () => {
  it("returns empty array for empty input", () => {
    expect(toYearlyCostData([])).toEqual([]);
  });

  it("aggregates costs within a single year", () => {
    const segments = [
      makeSegment({ end_date: "2025-03-05", cost: 30 }),
      makeSegment({ end_date: "2025-09-20", cost: 45 }),
    ];
    const result = toYearlyCostData(segments);
    expect(result).toHaveLength(1);
    expect(result[0].month).toBe("2025");
    expect(result[0].value).toBe(75);
  });

  it("returns separate entries for different years", () => {
    const segments = [
      makeSegment({ end_date: "2024-06-10", cost: 50 }),
      makeSegment({ end_date: "2025-01-10", cost: 60 }),
      makeSegment({ end_date: "2023-11-10", cost: 40 }),
    ];
    const result = toYearlyCostData(segments);
    expect(result).toHaveLength(3);
    expect(result[0].month).toBe("2023");
    expect(result[1].month).toBe("2024");
    expect(result[2].month).toBe("2025");
  });

  it("sorts years chronologically regardless of input order", () => {
    const segments = [
      makeSegment({ end_date: "2026-01-01", cost: 10 }),
      makeSegment({ end_date: "2024-01-01", cost: 20 }),
    ];
    const result = toYearlyCostData(segments);
    expect(result[0].month).toBe("2024");
    expect(result[1].month).toBe("2026");
  });
});

// ── toMonthlyDistanceData ───────────────────────────────

describe("toMonthlyDistanceData", () => {
  it("returns empty array for empty input", () => {
    expect(toMonthlyDistanceData([])).toEqual([]);
  });

  it("aggregates distances within a single month", () => {
    const segments = [
      makeSegment({
        start_date: "2025-03-01",
        end_date: "2025-03-05",
        distance: 300,
      }),
      makeSegment({
        start_date: "2025-03-05",
        end_date: "2025-03-20",
        distance: 450,
      }),
    ];
    const result = toMonthlyDistanceData(segments);
    expect(result).toHaveLength(1);
    expect(result[0].month).toBe("2025-03");
    expect(result[0].value).toBe(750);
  });

  it("distributes distance across months a segment spans", () => {
    // Segment from Jan 1 to Mar 1 = 59 days, 590 km
    // Jan: 30 days, Feb: 28 days, Mar: 1 day
    const segments = [
      makeSegment({
        start_date: "2025-01-01",
        end_date: "2025-03-01",
        distance: 590,
      }),
    ];
    const result = toMonthlyDistanceData(segments);
    expect(result).toHaveLength(3);
    expect(result[0].month).toBe("2025-01");
    expect(result[1].month).toBe("2025-02");
    expect(result[2].month).toBe("2025-03");
    // All months should have some distance (not zero)
    expect(result[0].value).toBeGreaterThan(0);
    expect(result[1].value).toBeGreaterThan(0);
    // Total should roughly equal original distance
    const total = result.reduce((sum, d) => sum + d.value, 0);
    expect(total).toBeCloseTo(590, 0);
  });

  it("fills gap months with zero when no segment spans them", () => {
    // Two separate segments with a gap month in between
    const segments = [
      makeSegment({
        start_date: "2025-01-01",
        end_date: "2025-01-15",
        distance: 500,
      }),
      makeSegment({
        start_date: "2025-03-01",
        end_date: "2025-03-15",
        distance: 600,
      }),
    ];
    const result = toMonthlyDistanceData(segments);
    expect(result).toHaveLength(3);
    expect(result[0].month).toBe("2025-01");
    expect(result[0].value).toBe(500);
    expect(result[1].month).toBe("2025-02");
    expect(result[1].value).toBe(0);
    expect(result[2].month).toBe("2025-03");
    expect(result[2].value).toBe(600);
  });
});

// ── toFuelPriceData ─────────────────────────────────────

describe("toFuelPriceData", () => {
  it("returns empty array for empty input", () => {
    expect(toFuelPriceData([])).toEqual([]);
  });

  it("excludes segments with zero fuel", () => {
    const segments = [
      makeSegment({ fuel: 40, cost: 60 }),
      makeSegment({ fuel: 0, cost: 0, end_date: "2025-02-01" }),
    ];
    const result = toFuelPriceData(segments);
    expect(result).toHaveLength(1);
  });

  it("calculates cost / fuel correctly", () => {
    const result = toFuelPriceData([makeSegment({ fuel: 40, cost: 80 })]);
    expect(result[0].value).toBe(2); // 80 / 40
  });

  it("parses dates correctly", () => {
    const result = toFuelPriceData([
      makeSegment({ end_date: "2025-08-15", fuel: 30, cost: 45 }),
    ]);
    expect(result[0].date.getFullYear()).toBe(2025);
    expect(result[0].date.getMonth()).toBe(7); // August = 7
    expect(result[0].value).toBe(1.5);
  });
});

// ── toSparklineData ─────────────────────────────────────

describe("toSparklineData", () => {
  it("returns empty array for empty input", () => {
    expect(toSparklineData([], (s) => s.efficiency)).toEqual([]);
  });

  it("uses index-based x values", () => {
    const segments = [
      makeSegment({ efficiency: 12 }),
      makeSegment({ efficiency: 14 }),
      makeSegment({ efficiency: 13 }),
    ];
    const result = toSparklineData(segments, (s) => s.efficiency);
    expect(result).toHaveLength(3);
    expect(result[0].x).toBe(0);
    expect(result[1].x).toBe(1);
    expect(result[2].x).toBe(2);
  });

  it("applies the accessor function correctly", () => {
    const segments = [
      makeSegment({ cost_per_distance: 0.12 }),
      makeSegment({ cost_per_distance: 0.15 }),
    ];
    const result = toSparklineData(segments, (s) => s.cost_per_distance);
    expect(result[0].y).toBe(0.12);
    expect(result[1].y).toBe(0.15);
  });

  it("works with computed accessor (cost / fuel)", () => {
    const segments = [
      makeSegment({ cost: 60, fuel: 40 }),
      makeSegment({ cost: 90, fuel: 30 }),
    ];
    const result = toSparklineData(segments, (s) => s.cost / s.fuel);
    expect(result[0].y).toBe(1.5);
    expect(result[1].y).toBe(3);
  });
});
