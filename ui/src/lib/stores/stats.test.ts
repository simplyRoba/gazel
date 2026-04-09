import { beforeEach, describe, expect, it, vi } from "vitest";
import type { VehicleStats, SegmentHistory } from "$lib/api";
import { ApiError } from "$lib/api";

vi.mock("$lib/api", async (importOriginal) => {
  const actual = await importOriginal<typeof import("$lib/api")>();
  return {
    ...actual,
    fetchVehicleStats: vi.fn(),
    fetchVehicleStatsHistory: vi.fn(),
  };
});

import * as api from "$lib/api";

const mockStats: VehicleStats = {
  total_distance: 1500.0,
  total_fuel: 120.5,
  total_cost: 250.0,
  fill_up_count: 5,
  average_efficiency: 12.45,
  average_cost_per_distance: 0.17,
  distance_unit: "km",
  volume_unit: "l",
  currency: "EUR",
};

const mockStats2: VehicleStats = {
  ...mockStats,
  total_cost: 300.0,
  average_efficiency: 10.2,
};

const mockHistory: SegmentHistory[] = [
  {
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
  },
  {
    start_date: "2026-02-01",
    end_date: "2026-03-01",
    start_odometer: 10500,
    end_odometer: 11000,
    distance: 500,
    fuel: 38.0,
    cost: 72.0,
    efficiency: 13.16,
    cost_per_distance: 0.14,
    is_valid: true,
    distance_unit: "km",
    volume_unit: "l",
    currency: "EUR",
  },
];

describe("stats store", () => {
  let store: typeof import("./stats.svelte");

  beforeEach(async () => {
    vi.clearAllMocks();
    vi.resetModules();
    store = await import("./stats.svelte");
  });

  describe("initial state", () => {
    it("has empty defaults", () => {
      expect(store.getVehicleStats(1)).toBeUndefined();
      expect(store.getVehicleHistory(1)).toEqual([]);
      expect(store.getLoading()).toBe(false);
      expect(store.getError()).toBeNull();
    });
  });

  describe("loadStats", () => {
    it("fetches and caches stats and history for a vehicle", async () => {
      vi.mocked(api.fetchVehicleStats).mockResolvedValue(mockStats);
      vi.mocked(api.fetchVehicleStatsHistory).mockResolvedValue(mockHistory);

      await store.loadStats(10);

      expect(api.fetchVehicleStats).toHaveBeenCalledWith(10);
      expect(api.fetchVehicleStatsHistory).toHaveBeenCalledWith(10);
      expect(store.getVehicleStats(10)).toEqual(mockStats);
      expect(store.getVehicleHistory(10)).toEqual(mockHistory);
      expect(store.getLoading()).toBe(false);
      expect(store.getError()).toBeNull();
    });

    it("sets error on failure", async () => {
      vi.mocked(api.fetchVehicleStats).mockRejectedValue(
        new ApiError(404, "VEHICLE_NOT_FOUND", "Vehicle not found."),
      );
      vi.mocked(api.fetchVehicleStatsHistory).mockRejectedValue(
        new ApiError(404, "VEHICLE_NOT_FOUND", "Vehicle not found."),
      );

      await store.loadStats(999);

      expect(store.getError()).toBe("Vehicle not found.");
      expect(store.getVehicleStats(999)).toBeUndefined();
      expect(store.getLoading()).toBe(false);
    });

    it("uses fallback message for non-ApiError", async () => {
      vi.mocked(api.fetchVehicleStats).mockRejectedValue(new Error("Network"));
      vi.mocked(api.fetchVehicleStatsHistory).mockRejectedValue(
        new Error("Network"),
      );

      await store.loadStats(10);

      expect(store.getError()).toBe("Failed to load stats");
    });
  });

  describe("loadAllStats", () => {
    it("fetches stats for all vehicles in parallel", async () => {
      vi.mocked(api.fetchVehicleStats)
        .mockResolvedValueOnce(mockStats)
        .mockResolvedValueOnce(mockStats2);
      vi.mocked(api.fetchVehicleStatsHistory)
        .mockResolvedValueOnce(mockHistory)
        .mockResolvedValueOnce([]);

      await store.loadAllStats([10, 20]);

      expect(api.fetchVehicleStats).toHaveBeenCalledTimes(2);
      expect(api.fetchVehicleStatsHistory).toHaveBeenCalledTimes(2);
      expect(store.getVehicleStats(10)).toEqual(mockStats);
      expect(store.getVehicleStats(20)).toEqual(mockStats2);
      expect(store.getVehicleHistory(10)).toEqual(mockHistory);
      expect(store.getVehicleHistory(20)).toEqual([]);
      expect(store.getLoading()).toBe(false);
    });
  });

  describe("invalidateStats", () => {
    it("clears cache and refetches", async () => {
      // First populate cache
      vi.mocked(api.fetchVehicleStats).mockResolvedValue(mockStats);
      vi.mocked(api.fetchVehicleStatsHistory).mockResolvedValue(mockHistory);
      await store.loadStats(10);
      expect(store.getVehicleStats(10)).toEqual(mockStats);

      // Now invalidate
      const updatedStats = { ...mockStats, total_cost: 999 };
      vi.mocked(api.fetchVehicleStats).mockResolvedValue(updatedStats);
      vi.mocked(api.fetchVehicleStatsHistory).mockResolvedValue([]);

      await store.invalidateStats(10);

      expect(store.getVehicleStats(10)).toEqual(updatedStats);
      expect(store.getVehicleHistory(10)).toEqual([]);
    });
  });

  describe("error clearing", () => {
    it("clears previous error on new loadStats call", async () => {
      vi.mocked(api.fetchVehicleStats).mockRejectedValue(
        new ApiError(500, "INTERNAL_ERROR", "First error"),
      );
      vi.mocked(api.fetchVehicleStatsHistory).mockRejectedValue(
        new ApiError(500, "INTERNAL_ERROR", "First error"),
      );
      await store.loadStats(10);
      expect(store.getError()).toBe("First error");

      vi.mocked(api.fetchVehicleStats).mockResolvedValue(mockStats);
      vi.mocked(api.fetchVehicleStatsHistory).mockResolvedValue(mockHistory);
      await store.loadStats(10);
      expect(store.getError()).toBeNull();
    });
  });
});
