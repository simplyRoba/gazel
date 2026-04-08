import { beforeEach, describe, expect, it, vi } from "vitest";
import type { Vehicle } from "$lib/api";
import { ApiError } from "$lib/api";

vi.mock("$lib/api", async (importOriginal) => {
  const actual = await importOriginal<typeof import("$lib/api")>();
  return {
    ...actual,
    fetchVehicles: vi.fn(),
    fetchVehicle: vi.fn(),
    createVehicle: vi.fn(),
    updateVehicle: vi.fn(),
    deleteVehicle: vi.fn(),
  };
});

import * as api from "$lib/api";

const mockVehicle: Vehicle = {
  id: 1,
  name: "Civic",
  make: "Honda",
  model: "Civic",
  year: 2024,
  fuel_type: "gasoline",
  notes: null,
  created_at: "2026-01-01T00:00:00Z",
  updated_at: "2026-01-01T00:00:00Z",
};

const mockVehicle2: Vehicle = {
  ...mockVehicle,
  id: 2,
  name: "Tacoma",
  make: "Toyota",
  model: "Tacoma",
};

describe("vehicle store", () => {
  // Re-import fresh module state for each test
  let store: typeof import("./vehicles.svelte");

  beforeEach(async () => {
    vi.clearAllMocks();
    vi.resetModules();
    store = await import("./vehicles.svelte");
  });

  describe("loadVehicles", () => {
    it("sets vehicle list on success", async () => {
      vi.mocked(api.fetchVehicles).mockResolvedValue([
        mockVehicle,
        mockVehicle2,
      ]);
      await store.loadVehicles();
      expect(store.getVehicles()).toEqual([mockVehicle, mockVehicle2]);
      expect(store.getError()).toBeNull();
      expect(store.getLoading()).toBe(false);
    });

    it("sets error on failure", async () => {
      vi.mocked(api.fetchVehicles).mockRejectedValue(
        new ApiError(500, "INTERNAL_ERROR", "An unexpected error occurred."),
      );
      await store.loadVehicles();
      expect(store.getVehicles()).toEqual([]);
      expect(store.getError()).toBe("An unexpected error occurred.");
    });

    it("sets fallback error for non-ApiError", async () => {
      vi.mocked(api.fetchVehicles).mockRejectedValue(
        new Error("Network error"),
      );
      await store.loadVehicles();
      expect(store.getError()).toBe("Failed to load vehicles");
    });
  });

  describe("createVehicle", () => {
    it("appends vehicle to list on success", async () => {
      vi.mocked(api.fetchVehicles).mockResolvedValue([mockVehicle]);
      await store.loadVehicles();

      vi.mocked(api.createVehicle).mockResolvedValue(mockVehicle2);
      const result = await store.createVehicle({ name: "Tacoma" });

      expect(result).toEqual(mockVehicle2);
      expect(store.getVehicles()).toEqual([mockVehicle, mockVehicle2]);
      expect(store.getError()).toBeNull();
    });

    it("returns null and sets error on failure", async () => {
      vi.mocked(api.createVehicle).mockRejectedValue(
        new ApiError(422, "VEHICLE_NAME_REQUIRED", "Vehicle name is required."),
      );
      const result = await store.createVehicle({ name: "" });

      expect(result).toBeNull();
      expect(store.getError()).toBe("Vehicle name is required.");
    });
  });

  describe("updateVehicle", () => {
    it("replaces vehicle in list on success", async () => {
      vi.mocked(api.fetchVehicles).mockResolvedValue([
        mockVehicle,
        mockVehicle2,
      ]);
      await store.loadVehicles();

      const updated = { ...mockVehicle, name: "Updated Civic" };
      vi.mocked(api.updateVehicle).mockResolvedValue(updated);
      const result = await store.updateVehicle(1, { name: "Updated Civic" });

      expect(result).toEqual(updated);
      expect(store.getVehicles()[0].name).toBe("Updated Civic");
      expect(store.getVehicles()[1]).toEqual(mockVehicle2);
    });

    it("returns null and sets error on failure", async () => {
      vi.mocked(api.updateVehicle).mockRejectedValue(
        new ApiError(500, "INTERNAL_ERROR", "An unexpected error occurred."),
      );
      const result = await store.updateVehicle(1, { name: "" });

      expect(result).toBeNull();
      expect(store.getError()).toBe("An unexpected error occurred.");
    });
  });

  describe("deleteVehicle", () => {
    it("removes vehicle from list on success", async () => {
      vi.mocked(api.fetchVehicles).mockResolvedValue([
        mockVehicle,
        mockVehicle2,
      ]);
      await store.loadVehicles();

      vi.mocked(api.deleteVehicle).mockResolvedValue(undefined);
      const result = await store.deleteVehicle(1);

      expect(result).toBe(true);
      expect(store.getVehicles()).toEqual([mockVehicle2]);
    });

    it("returns false and sets error on failure", async () => {
      vi.mocked(api.deleteVehicle).mockRejectedValue(
        new Error("Delete failed"),
      );
      const result = await store.deleteVehicle(1);

      expect(result).toBe(false);
      expect(store.getError()).toBe("Failed to delete vehicle");
    });
  });

  describe("error clearing", () => {
    it("clears previous error on new action", async () => {
      // Fail first
      vi.mocked(api.fetchVehicles).mockRejectedValue(
        new ApiError(500, "INTERNAL_ERROR", "First error"),
      );
      await store.loadVehicles();
      expect(store.getError()).toBe("First error");

      // Succeed — error should be cleared
      vi.mocked(api.fetchVehicles).mockResolvedValue([mockVehicle]);
      await store.loadVehicles();
      expect(store.getError()).toBeNull();
    });
  });
});
