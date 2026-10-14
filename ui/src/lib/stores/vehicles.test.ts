import { beforeEach, describe, expect, it, vi } from "vitest";
import type { StatsResponse, Vehicle } from "#lib/api.js";
import { ApiError } from "#lib/api.js";

vi.mock("#lib/api.js", async (importOriginal) => {
  const actual = await importOriginal<typeof import("#lib/api.js")>();
  return {
    ...actual,
    fetchVehicles: vi.fn(),
    fetchVehicle: vi.fn(),
    fetchFillups: vi.fn(),
    createVehicle: vi.fn(),
    updateVehicle: vi.fn(),
    deleteVehicle: vi.fn(),
    fetchStats: vi.fn(),
  };
});

vi.mock("#lib/stores/settings.svelte.js", () => ({
  getSettings: () => ({
    unit_system: "metric",
    distance_unit: "km",
    volume_unit: "l",
    currency: "USD",
    color_mode: "system",
    locale: "en",
  }),
}));

vi.mock("#lib/stores/notifications.svelte.js", () => ({
  pushNotification: vi.fn(),
}));

import * as api from "#lib/api.js";
import { pushNotification } from "#lib/stores/notifications.svelte.js";

function deferred<T>() {
  let resolve!: (value: T) => void;
  let reject!: (reason: unknown) => void;
  const promise = new Promise<T>((yes, no) => {
    resolve = yes;
    reject = no;
  });
  return { promise, resolve, reject };
}

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

const statsResponse: StatsResponse = {
  units: { distance_unit: "km", volume_unit: "l", currency: "EUR" },
  fleet: {
    total_distance: 0,
    total_fuel: 0,
    total_cost: 0,
    fill_up_count: 0,
    average_efficiency: null,
    cost_per_distance: null,
    cost_per_volume: null,
  },
  vehicles: [],
  fleet_efficiency: { points: [], source_count: 0, reduced: false },
  selected: null,
};

describe("vehicle store", () => {
  // Re-import fresh module state for each test
  let store: typeof import("./vehicles.svelte");

  beforeEach(async () => {
    vi.resetAllMocks();
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

    it("sets error on failure with API error", async () => {
      vi.mocked(api.fetchVehicles).mockRejectedValue(
        new ApiError(500, "INTERNAL_ERROR", "An unexpected error occurred."),
      );
      await store.loadVehicles();
      expect(store.getVehicles()).toEqual([]);
      // resolveError maps INTERNAL_ERROR via t() to the en.json value
      expect(store.getError()).toBe("An unexpected error occurred.");
    });

    it("sets fallback error for non-ApiError", async () => {
      vi.mocked(api.fetchVehicles).mockRejectedValue(
        new Error("Network error"),
      );
      await store.loadVehicles();
      // t('store.vehicles.loadFailed') returns the en.json value
      expect(store.getError()).toBe("Failed to load vehicles");
    });
  });

  describe("read ordering", () => {
    it.each(["success", "failure"])(
      "waits for the latest fleet and ignores stale %s while it is pending",
      async (outcome) => {
        const old = deferred<Vehicle[]>();
        const next = deferred<Vehicle[]>();
        vi.mocked(api.fetchVehicles)
          .mockReturnValueOnce(old.promise)
          .mockReturnValueOnce(next.promise);
        let initialized = false;
        const first: Promise<void> = store.loadVehicles().then(() => {
          initialized = true;
          expect(store.getVehicles()).toEqual([mockVehicle2]);
        });
        const second = store.loadVehicles();
        if (outcome === "success") old.resolve([mockVehicle]);
        else old.reject(new Error("old failure"));
        await Promise.resolve();
        await Promise.resolve();
        expect(initialized).toBe(false);
        expect(store.getLoading()).toBe(true);
        expect(store.getVehicles()).toEqual([]);
        expect(store.getError()).toBeNull();
        expect(pushNotification).not.toHaveBeenCalled();
        next.resolve([mockVehicle2]);
        await Promise.all([first, second]);
        expect(initialized).toBe(true);
        expect(store.getLoading()).toBe(false);
      },
    );

    it.each(["success", "failure"])(
      "ignores stale %s after a newer failure without changing its error or notification",
      async (outcome) => {
        const old = deferred<Vehicle[]>();
        vi.mocked(api.fetchVehicles)
          .mockReturnValueOnce(old.promise)
          .mockRejectedValueOnce(
            new ApiError(500, "INTERNAL_ERROR", "new failure"),
          );
        const first = store.loadVehicles();
        await store.loadVehicles();
        const error = store.getError();
        expect(error).toBe("An unexpected error occurred.");
        if (outcome === "success") old.resolve([mockVehicle]);
        else old.reject(new Error("old failure"));
        await first;
        expect(store.getVehicles()).toEqual([]);
        expect(store.getError()).toBe(error);
        expect(store.getLoading()).toBe(false);
        expect(pushNotification).toHaveBeenCalledTimes(1);
      },
    );

    it.each(["success", "failure"])(
      "ignores stale %s after a newer successful fleet",
      async (outcome) => {
        const old = deferred<Vehicle[]>();
        vi.mocked(api.fetchVehicles)
          .mockReturnValueOnce(old.promise)
          .mockResolvedValueOnce([mockVehicle2]);
        const first = store.loadVehicles();
        await store.loadVehicles();
        if (outcome === "success") old.resolve([mockVehicle]);
        else old.reject(new Error("old failure"));
        await first;
        expect(store.getVehicles()).toEqual([mockVehicle2]);
        expect(store.getError()).toBeNull();
        expect(store.getLoading()).toBe(false);
        expect(pushNotification).not.toHaveBeenCalled();
      },
    );

    it.each(["create", "update", "delete"])(
      "does not undo committed %s with a pending list response",
      async (kind) => {
        vi.mocked(api.fetchVehicles).mockResolvedValueOnce([mockVehicle]);
        await store.loadVehicles();
        const old = deferred<Vehicle[]>();
        vi.mocked(api.fetchVehicles).mockReturnValueOnce(old.promise);
        const pending = store.loadVehicles();
        const updated = { ...mockVehicle, name: "Updated" };
        vi.mocked(api.createVehicle).mockResolvedValue(mockVehicle2);
        vi.mocked(api.updateVehicle).mockResolvedValue(updated);
        vi.mocked(api.deleteVehicle).mockResolvedValue(undefined);
        if (kind === "create") await store.createVehicle({ name: "Tacoma" });
        else if (kind === "update")
          await store.updateVehicle(1, { name: "Updated" });
        else await store.deleteVehicle(1);
        const expected =
          kind === "create"
            ? [mockVehicle, mockVehicle2]
            : kind === "update"
              ? [updated]
              : [];
        expect(store.getLoading()).toBe(false);
        old.resolve([mockVehicle]);
        await pending;
        expect(store.getVehicles()).toEqual(expected);
        expect(store.getError()).toBeNull();
        expect(pushNotification).not.toHaveBeenCalled();
        vi.mocked(api.fetchVehicles).mockResolvedValueOnce([mockVehicle2]);
        await store.loadVehicles();
        expect(store.getVehicles()).toEqual([mockVehicle2]);
      },
    );
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

  describe("statistics refresh hooks", () => {
    it.each(["create", "update", "delete"])(
      "refreshes once after successful %s commits",
      async (kind) => {
        const stats = await import("./stats.svelte");
        vi.mocked(api.fetchStats).mockResolvedValue(statsResponse);
        await stats.loadStats(2);
        vi.mocked(api.fetchStats).mockClear();
        vi.mocked(api.fetchVehicles).mockResolvedValue([
          mockVehicle,
          mockVehicle2,
        ]);
        await store.loadVehicles();
        vi.mocked(api.createVehicle).mockResolvedValue(mockVehicle);
        vi.mocked(api.updateVehicle).mockResolvedValue(mockVehicle);
        vi.mocked(api.deleteVehicle).mockResolvedValue(undefined);
        if (kind === "create") await store.createVehicle({ name: "Civic" });
        else if (kind === "update")
          await store.updateVehicle(1, { name: "Civic" });
        else await store.deleteVehicle(1);
        expect(api.fetchStats).toHaveBeenCalledExactlyOnceWith(2);
      },
    );

    it.each(["create", "update", "delete"])(
      "does not refresh failed %s",
      async (kind) => {
        const stats = await import("./stats.svelte");
        vi.mocked(api.fetchStats).mockResolvedValue(statsResponse);
        await stats.loadStats(1);
        vi.mocked(api.fetchStats).mockClear();
        vi.mocked(api.createVehicle).mockRejectedValue(new Error("failed"));
        vi.mocked(api.updateVehicle).mockRejectedValue(new Error("failed"));
        vi.mocked(api.deleteVehicle).mockRejectedValue(new Error("failed"));
        if (kind === "create") await store.createVehicle({ name: "Civic" });
        else if (kind === "update")
          await store.updateVehicle(1, { name: "Civic" });
        else await store.deleteVehicle(1);
        expect(api.fetchStats).not.toHaveBeenCalled();
        expect(stats.getSelectedVehicleId()).toBe(1);
      },
    );

    it.each([true, false])(
      "retargets a deleted selection in one request (remaining vehicle: %s)",
      async (hasRemaining) => {
        const stats = await import("./stats.svelte");
        vi.mocked(api.fetchStats).mockResolvedValue(statsResponse);
        await stats.loadStats(1);
        vi.mocked(api.fetchStats).mockClear();
        const fillups = await import("./fillups.svelte");
        vi.mocked(api.fetchFillups).mockResolvedValue({
          items: [],
          next_cursor: null,
          efficiency: { distance_unit: "km", volume_unit: "l", values: [] },
        });
        await fillups.setActiveVehicle(1);
        vi.mocked(api.fetchFillups).mockClear();
        vi.mocked(api.fetchVehicles).mockResolvedValue(
          hasRemaining ? [mockVehicle, mockVehicle2] : [mockVehicle],
        );
        await store.loadVehicles();
        vi.mocked(api.deleteVehicle).mockResolvedValue(undefined);
        await store.deleteVehicle(1);
        expect(api.fetchStats).toHaveBeenCalledExactlyOnceWith(
          hasRemaining ? 2 : null,
        );
        expect(stats.getSelectedVehicleId()).toBe(hasRemaining ? 2 : null);
        expect(fillups.getActiveVehicleId()).toBe(hasRemaining ? 2 : null);
        expect(vi.mocked(api.fetchFillups).mock.calls).toEqual(
          hasRemaining ? [[2]] : [],
        );
      },
    );

    it("does not reload the list when deleting a nonselected vehicle", async () => {
      const fillups = await import("./fillups.svelte");
      const stats = await import("./stats.svelte");
      vi.mocked(api.fetchFillups).mockResolvedValue({
        items: [],
        next_cursor: null,
        efficiency: { distance_unit: "km", volume_unit: "l", values: [] },
      });
      vi.mocked(api.fetchStats).mockResolvedValue(statsResponse);
      vi.mocked(api.fetchVehicles).mockResolvedValue([
        mockVehicle,
        mockVehicle2,
      ]);
      await store.loadVehicles();
      await fillups.setActiveVehicle(2);
      await stats.loadStats(2);
      vi.mocked(api.fetchFillups).mockClear();
      vi.mocked(api.fetchStats).mockClear();
      vi.mocked(api.deleteVehicle).mockResolvedValue(undefined);
      await store.deleteVehicle(1);
      expect(fillups.getActiveVehicleId()).toBe(2);
      expect(api.fetchFillups).not.toHaveBeenCalled();
      expect(api.fetchStats).toHaveBeenCalledExactlyOnceWith(2);
    });

    it("keeps all mutations dormant before the dashboard first requests statistics", async () => {
      vi.mocked(api.createVehicle).mockResolvedValue(mockVehicle);
      vi.mocked(api.updateVehicle).mockResolvedValue(mockVehicle);
      vi.mocked(api.deleteVehicle).mockResolvedValue(undefined);
      await store.createVehicle({ name: "Civic" });
      await store.updateVehicle(1, { name: "Civic" });
      await store.deleteVehicle(1);
      expect(api.fetchStats).not.toHaveBeenCalled();
    });

    it("does not refresh before a mutation commits", async () => {
      const stats = await import("./stats.svelte");
      vi.mocked(api.fetchStats).mockResolvedValue(statsResponse);
      await stats.loadStats();
      vi.mocked(api.fetchStats).mockClear();
      let resolve!: (vehicle: Vehicle) => void;
      vi.mocked(api.createVehicle).mockReturnValue(
        new Promise((res) => {
          resolve = res;
        }),
      );
      const pending = store.createVehicle({ name: "Civic" });
      expect(api.fetchStats).not.toHaveBeenCalled();
      resolve(mockVehicle);
      await pending;
      expect(api.fetchStats).toHaveBeenCalledExactlyOnceWith(null);
    });
  });

  describe("error clearing", () => {
    it("clears previous error on new action", async () => {
      // Fail first
      vi.mocked(api.fetchVehicles).mockRejectedValue(
        new ApiError(500, "INTERNAL_ERROR", "First error"),
      );
      await store.loadVehicles();
      // resolveError sees 'INTERNAL_ERROR' and returns the en.json translation
      expect(store.getError()).toBe("An unexpected error occurred.");

      // Succeed — error should be cleared
      vi.mocked(api.fetchVehicles).mockResolvedValue([mockVehicle]);
      await store.loadVehicles();
      expect(store.getError()).toBeNull();
    });
  });
});
