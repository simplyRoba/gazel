import { beforeEach, describe, expect, it, vi } from "vitest";
import type { Fillup } from "$lib/api";
import { ApiError } from "$lib/api";

vi.mock("$lib/api", async (importOriginal) => {
  const actual = await importOriginal<typeof import("$lib/api")>();
  return {
    ...actual,
    fetchFillups: vi.fn(),
    fetchFillup: vi.fn(),
    createFillup: vi.fn(),
    updateFillup: vi.fn(),
    deleteFillup: vi.fn(),
  };
});

import * as api from "$lib/api";

const mockFillup: Fillup = {
  id: 1,
  vehicle_id: 10,
  date: "2026-03-01",
  odometer: 10000,
  fuel_amount: 42.3,
  fuel_unit: "l",
  cost: 78.5,
  currency: "EUR",
  is_full_tank: true,
  is_missed: false,
  station: "Shell",
  notes: null,
  created_at: "2026-03-01T10:00:00Z",
  updated_at: "2026-03-01T10:00:00Z",
};

const mockFillup2: Fillup = {
  ...mockFillup,
  id: 2,
  date: "2026-04-01",
  odometer: 10500,
  fuel_amount: 38.1,
  cost: 70.2,
  station: "BP",
};

describe("fillup store", () => {
  let store: typeof import("./fillups.svelte");

  beforeEach(async () => {
    vi.clearAllMocks();
    vi.resetModules();
    store = await import("./fillups.svelte");
  });

  describe("initial state", () => {
    it("has empty defaults", () => {
      expect(store.getFillups()).toEqual([]);
      expect(store.getLoading()).toBe(false);
      expect(store.getError()).toBeNull();
      expect(store.getActiveVehicleId()).toBeNull();
    });
  });

  describe("loadFillups", () => {
    it("populates cache for vehicle on success", async () => {
      vi.mocked(api.fetchFillups).mockResolvedValue([mockFillup2, mockFillup]);
      store.setActiveVehicle(10);
      // setActiveVehicle triggers loadFillups; wait for it
      await vi.waitFor(() => {
        expect(store.getFillups()).toEqual([mockFillup2, mockFillup]);
      });
      expect(store.getError()).toBeNull();
      expect(store.getLoading()).toBe(false);
    });

    it("sets error on failure", async () => {
      vi.mocked(api.fetchFillups).mockRejectedValue(
        new ApiError(500, "INTERNAL_ERROR", "Database error"),
      );
      await store.loadFillups(10);
      expect(store.getError()).toBe("Database error");
    });

    it("uses fallback message for non-ApiError", async () => {
      vi.mocked(api.fetchFillups).mockRejectedValue(new Error("Network"));
      await store.loadFillups(10);
      expect(store.getError()).toBe("Failed to load fill-ups");
    });
  });

  describe("createFillup", () => {
    it("adds fill-up to cache in sort order", async () => {
      vi.mocked(api.fetchFillups).mockResolvedValue([mockFillup]);
      await store.loadFillups(10);

      vi.mocked(api.createFillup).mockResolvedValue(mockFillup2);
      const result = await store.createFillup(10, {
        date: "2026-04-01",
        odometer: 10500,
        fuel_amount: 38.1,
        cost: 70.2,
      });

      expect(result).toEqual(mockFillup2);
      const fillups = store.getFillupsByVehicle(10);
      // Newer fill-up should be first (date desc)
      expect(fillups[0].id).toBe(2);
      expect(fillups[1].id).toBe(1);
    });

    it("returns null and sets error on failure", async () => {
      vi.mocked(api.createFillup).mockRejectedValue(
        new ApiError(422, "FILLUP_ODOMETER_REQUIRED", "Odometer required"),
      );
      const result = await store.createFillup(10, {
        date: "2026-04-01",
        odometer: 0,
        fuel_amount: 30,
        cost: 50,
      });
      expect(result).toBeNull();
      expect(store.getError()).toBe("Odometer required");
    });
  });

  describe("updateFillup", () => {
    it("replaces fill-up in cache", async () => {
      vi.mocked(api.fetchFillups).mockResolvedValue([mockFillup2, mockFillup]);
      await store.loadFillups(10);

      const updated = { ...mockFillup, cost: 80.0 };
      vi.mocked(api.updateFillup).mockResolvedValue(updated);
      const result = await store.updateFillup(10, 1, {
        date: "2026-03-01",
        odometer: 10000,
        fuel_amount: 42.3,
        cost: 80.0,
      });

      expect(result).toEqual(updated);
      expect(store.getFillupsByVehicle(10).find((f) => f.id === 1)?.cost).toBe(
        80.0,
      );
    });

    it("returns null and sets error on failure", async () => {
      vi.mocked(api.updateFillup).mockRejectedValue(new Error("fail"));
      const result = await store.updateFillup(10, 1, {
        date: "2026-03-01",
        odometer: 10000,
        fuel_amount: 42.3,
        cost: 80.0,
      });
      expect(result).toBeNull();
      expect(store.getError()).toBe("Failed to update fill-up");
    });
  });

  describe("deleteFillup", () => {
    it("removes fill-up from cache", async () => {
      vi.mocked(api.fetchFillups).mockResolvedValue([mockFillup2, mockFillup]);
      await store.loadFillups(10);

      vi.mocked(api.deleteFillup).mockResolvedValue(undefined);
      const result = await store.deleteFillup(10, 1);

      expect(result).toBe(true);
      expect(store.getFillupsByVehicle(10)).toEqual([mockFillup2]);
    });

    it("returns false and sets error on failure", async () => {
      vi.mocked(api.deleteFillup).mockRejectedValue(new Error("fail"));
      const result = await store.deleteFillup(10, 1);
      expect(result).toBe(false);
      expect(store.getError()).toBe("Failed to delete fill-up");
    });
  });

  describe("setActiveVehicle", () => {
    it("updates active vehicle id and triggers load", async () => {
      vi.mocked(api.fetchFillups).mockResolvedValue([mockFillup]);
      await store.setActiveVehicle(10);

      expect(store.getActiveVehicleId()).toBe(10);
      expect(api.fetchFillups).toHaveBeenCalledWith(10);
      expect(store.getFillups()).toEqual([mockFillup]);
    });
  });

  describe("error clearing", () => {
    it("clears previous error on new action", async () => {
      vi.mocked(api.fetchFillups).mockRejectedValue(
        new ApiError(500, "INTERNAL_ERROR", "First error"),
      );
      await store.loadFillups(10);
      expect(store.getError()).toBe("First error");

      vi.mocked(api.fetchFillups).mockResolvedValue([mockFillup]);
      await store.loadFillups(10);
      expect(store.getError()).toBeNull();
    });
  });

  describe("fetchFillup (single)", () => {
    it("fetches a single fill-up by id", async () => {
      vi.mocked(api.fetchFillup).mockResolvedValue(mockFillup);
      const result = await api.fetchFillup(10, 1);
      expect(api.fetchFillup).toHaveBeenCalledWith(10, 1);
      expect(result).toEqual(mockFillup);
    });
  });
});
