import { describe, expect, it } from "vitest";
import type { SegmentHistory, Fillup, VehicleStats, Vehicle } from "$lib/api";
import {
  buildEfficiencyMap,
  getEfficiencyForFillup,
  computeFleetSummary,
} from "$lib/stats";

// ── Test data ────────────────────────────────────────────

const segment = (overrides: Partial<SegmentHistory> = {}): SegmentHistory => ({
  start_date: "2026-01-15",
  end_date: "2026-02-01",
  start_odometer: 10000,
  end_odometer: 10500,
  distance: 500,
  fuel: 42.5,
  cost: 85.0,
  efficiency: 11.76,
  cost_per_distance: 0.17,
  is_valid: true,
  distance_unit: "km",
  volume_unit: "l",
  currency: "EUR",
  ...overrides,
});

const fillup = (overrides: Partial<Fillup> = {}): Fillup => ({
  id: 1,
  vehicle_id: 10,
  date: "2026-02-01",
  odometer: 10500,
  fuel_amount: 42.5,
  fuel_unit: "l",
  cost: 85.0,
  currency: "EUR",
  is_full_tank: true,
  is_missed: false,
  station: null,
  notes: null,
  created_at: "2026-02-01T10:00:00Z",
  updated_at: "2026-02-01T10:00:00Z",
  ...overrides,
});

const vehicle = (overrides: Partial<Vehicle> = {}): Vehicle => ({
  id: 1,
  name: "Car A",
  make: null,
  model: null,
  year: null,
  fuel_type: "gasoline",
  notes: null,
  created_at: "2026-01-01T00:00:00Z",
  updated_at: "2026-01-01T00:00:00Z",
  ...overrides,
});

const stats = (overrides: Partial<VehicleStats> = {}): VehicleStats => ({
  total_distance: 1500,
  total_fuel: 120.5,
  total_cost: 250,
  fill_up_count: 5,
  average_efficiency: 12.45,
  average_cost_per_distance: 0.17,
  distance_unit: "km",
  volume_unit: "l",
  currency: "EUR",
  ...overrides,
});

// ── buildEfficiencyMap ───────────────────────────────────

describe("buildEfficiencyMap", () => {
  it("maps valid segments by end_date and end_odometer", () => {
    const segments = [
      segment({
        end_date: "2026-02-01",
        end_odometer: 10500,
        efficiency: 11.76,
      }),
      segment({
        end_date: "2026-03-01",
        end_odometer: 11000,
        efficiency: 13.16,
      }),
    ];
    const map = buildEfficiencyMap(segments);

    expect(map.size).toBe(2);
    expect(map.get("2026-02-01|10500")).toBe(11.76);
    expect(map.get("2026-03-01|11000")).toBe(13.16);
  });

  it("excludes invalid segments", () => {
    const segments = [
      segment({ is_valid: true, efficiency: 11.76 }),
      segment({
        end_date: "2026-03-01",
        end_odometer: 11000,
        is_valid: false,
        efficiency: 8.0,
      }),
    ];
    const map = buildEfficiencyMap(segments);

    expect(map.size).toBe(1);
    expect(map.has("2026-03-01|11000")).toBe(false);
  });

  it("returns empty map for empty segments", () => {
    const map = buildEfficiencyMap([]);
    expect(map.size).toBe(0);
  });
});

// ── getEfficiencyForFillup ───────────────────────────────

describe("getEfficiencyForFillup", () => {
  it("returns efficiency when fill-up matches a segment end", () => {
    const map = new Map([["2026-02-01|10500", 11.76]]);
    const f = fillup({ date: "2026-02-01", odometer: 10500 });

    expect(getEfficiencyForFillup(f, map)).toBe(11.76);
  });

  it("returns null when fill-up does not match any segment", () => {
    const map = new Map([["2026-02-01|10500", 11.76]]);
    const f = fillup({ date: "2026-02-15", odometer: 10700 });

    expect(getEfficiencyForFillup(f, map)).toBeNull();
  });

  it("returns null for empty map", () => {
    const f = fillup();
    expect(getEfficiencyForFillup(f, new Map())).toBeNull();
  });
});

// ── computeFleetSummary ──────────────────────────────────

describe("computeFleetSummary", () => {
  it("returns null when no vehicles", () => {
    expect(computeFleetSummary([], () => undefined)).toBeNull();
  });

  it("aggregates distance, fill-ups, and computes cost ratios", () => {
    const vehicles = [
      vehicle({ id: 1, name: "Car A" }),
      vehicle({ id: 2, name: "Car B" }),
    ];
    const statsMap = new Map([
      [
        1,
        stats({
          total_distance: 1500,
          total_fuel: 120,
          total_cost: 250,
          fill_up_count: 5,
        }),
      ],
      [
        2,
        stats({
          total_distance: 1000,
          total_fuel: 80,
          total_cost: 160,
          fill_up_count: 3,
        }),
      ],
    ]);
    const result = computeFleetSummary(vehicles, (id) => statsMap.get(id));

    expect(result).not.toBeNull();
    expect(result!.totalDistance).toBe(2500);
    expect(result!.totalFillups).toBe(8);
    expect(result!.costPerDistance).toBeCloseTo(410 / 2500);
    expect(result!.costPerVolume).toBeCloseTo(410 / 200);
  });

  it("returns null cost ratios when no distance or fuel", () => {
    const vehicles = [vehicle({ id: 1, name: "Car A" })];
    const statsMap = new Map([
      [
        1,
        stats({
          total_distance: 0,
          total_fuel: 0,
          total_cost: 0,
          fill_up_count: 0,
        }),
      ],
    ]);
    const result = computeFleetSummary(vehicles, (id) => statsMap.get(id));

    expect(result!.costPerDistance).toBeNull();
    expect(result!.costPerVolume).toBeNull();
  });

  it("returns zeroes when no stats loaded yet", () => {
    const vehicles = [
      vehicle({ id: 1, name: "Car A" }),
      vehicle({ id: 2, name: "Car B" }),
    ];
    const result = computeFleetSummary(vehicles, () => undefined);

    expect(result!.totalDistance).toBe(0);
    expect(result!.totalFillups).toBe(0);
    expect(result!.costPerDistance).toBeNull();
    expect(result!.costPerVolume).toBeNull();
  });

  it("works with a single vehicle", () => {
    const vehicles = [vehicle({ id: 1, name: "Car A" })];
    const statsMap = new Map([
      [
        1,
        stats({
          total_distance: 1500,
          total_fuel: 120,
          total_cost: 250,
          fill_up_count: 5,
        }),
      ],
    ]);
    const result = computeFleetSummary(vehicles, (id) => statsMap.get(id));

    expect(result!.totalDistance).toBe(1500);
    expect(result!.totalFillups).toBe(5);
    expect(result!.costPerDistance).toBeCloseTo(250 / 1500);
    expect(result!.costPerVolume).toBeCloseTo(250 / 120);
  });
});
