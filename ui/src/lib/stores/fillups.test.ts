import { beforeEach, describe, expect, it, vi } from "vitest";

import { ApiError, type Fillup, type FillupPage } from "$lib/api";

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

vi.mock("$lib/stores/settings.svelte", () => ({
  getSettings: () => ({
    unit_system: "metric",
    distance_unit: "km",
    volume_unit: "l",
    currency: "USD",
    color_mode: "system",
    locale: "en",
  }),
}));

import * as api from "$lib/api";

function fillup(id: number, overrides: Partial<Fillup> = {}): Fillup {
  return {
    id,
    vehicle_id: 10,
    date: `2026-03-${String(id).padStart(2, "0")}`,
    odometer: 10_000 + id,
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
    ...overrides,
  };
}

function page(items: Fillup[], next_cursor: string | null): FillupPage {
  return { items, next_cursor };
}

function deferred<T>() {
  let resolve!: (value: T) => void;
  let reject!: (reason?: unknown) => void;
  const promise = new Promise<T>((resolvePromise, rejectPromise) => {
    resolve = resolvePromise;
    reject = rejectPromise;
  });
  return { promise, resolve, reject };
}

describe("fillup store pagination", () => {
  let store: typeof import("./fillups.svelte");

  beforeEach(async () => {
    vi.clearAllMocks();
    vi.resetModules();
    store = await import("./fillups.svelte");
  });

  it("starts empty and exposes dashboard continuation state", () => {
    expect(store.getFillups()).toEqual([]);
    expect(store.getLoading()).toBe(false);
    expect(store.getLoadingMore()).toBe(false);
    expect(store.getContinuationError()).toBeNull();
    expect(store.getNextCursor()).toBeNull();
    expect(store.getActiveVehicleId()).toBeNull();
  });

  it("loads the first page into the active vehicle chain", async () => {
    vi.mocked(api.fetchFillups).mockResolvedValue(
      page([fillup(2), fillup(1)], "cursor-1"),
    );

    await store.setActiveVehicle(10);

    expect(api.fetchFillups).toHaveBeenCalledWith(10);
    expect(store.getFillups()).toEqual([fillup(2), fillup(1)]);
    expect(store.getNextCursor()).toBe("cursor-1");
    expect(store.getLoading()).toBe(false);
    expect(store.getFillupPageChain(10)).toMatchObject({
      items: [fillup(2), fillup(1)],
      nextCursor: "cursor-1",
      generation: 1,
    });
  });

  it("keeps completed vehicle chains independent while switching", async () => {
    vi.mocked(api.fetchFillups)
      .mockResolvedValueOnce(page([fillup(10, { vehicle_id: 1 })], "one-more"))
      .mockResolvedValueOnce(page([fillup(20, { vehicle_id: 2 })], null));

    await store.setActiveVehicle(1);
    await store.setActiveVehicle(2);

    expect(store.getActiveVehicleId()).toBe(2);
    expect(store.getFillups()).toEqual([fillup(20, { vehicle_id: 2 })]);
    expect(store.getFillupsByVehicle(1)).toEqual([
      fillup(10, { vehicle_id: 1 }),
    ]);
    expect(store.getFillupPageChain(1).nextCursor).toBe("one-more");
  });

  it.each(["success", "failure"])(
    "ignores stale initial-load %s after switching vehicles",
    async (outcome) => {
      const vehicleOne = deferred<FillupPage>();
      vi.mocked(api.fetchFillups)
        .mockReturnValueOnce(vehicleOne.promise)
        .mockResolvedValueOnce(page([fillup(20, { vehicle_id: 2 })], null));

      const loadOne = store.setActiveVehicle(1);
      await store.setActiveVehicle(2);
      if (outcome === "success") {
        vehicleOne.resolve(page([fillup(10, { vehicle_id: 1 })], "stale"));
      } else {
        vehicleOne.reject(new Error("stale failure"));
      }
      await loadOne;

      expect(store.getActiveVehicleId()).toBe(2);
      expect(store.getFillups()).toEqual([fillup(20, { vehicle_id: 2 })]);
      expect(store.getFillupsByVehicle(1)).toEqual([]);
      expect(store.getFillupPageChain(1)).toMatchObject({
        initialLoading: false,
        nextCursor: null,
      });
      expect(store.getError()).toBeNull();
    },
  );

  it.each(["success", "failure"])(
    "ignores stale continuation %s after switching vehicles",
    async (outcome) => {
      const continuation = deferred<FillupPage>();
      vi.mocked(api.fetchFillups)
        .mockResolvedValueOnce(
          page([fillup(10, { vehicle_id: 1 })], "vehicle-one-cursor"),
        )
        .mockReturnValueOnce(continuation.promise)
        .mockResolvedValueOnce(page([fillup(20, { vehicle_id: 2 })], null));

      await store.setActiveVehicle(1);
      const loadMore = store.loadMoreFillups();
      await store.setActiveVehicle(2);
      if (outcome === "success") {
        continuation.resolve(page([fillup(9, { vehicle_id: 1 })], null));
      } else {
        continuation.reject(new Error("stale failure"));
      }
      await loadMore;

      expect(store.getFillupsByVehicle(1)).toEqual([
        fillup(10, { vehicle_id: 1 }),
      ]);
      expect(store.getFillupPageChain(1)).toMatchObject({
        nextCursor: "vehicle-one-cursor",
        loadingMore: false,
        continuationError: null,
      });
      expect(store.getFillups()).toEqual([fillup(20, { vehicle_id: 2 })]);
      expect(store.getError()).toBeNull();
    },
  );

  it("appends only unseen continuation items in server order and replaces the cursor", async () => {
    vi.mocked(api.fetchFillups)
      .mockResolvedValueOnce(page([fillup(4), fillup(3)], "cursor-1"))
      .mockResolvedValueOnce(
        page([fillup(3), fillup(2), fillup(1)], "cursor-2"),
      );
    await store.setActiveVehicle(10);

    await store.loadMoreFillups();

    expect(api.fetchFillups).toHaveBeenLastCalledWith(10, "cursor-1");
    expect(store.getFillups().map(({ id }) => id)).toEqual([4, 3, 2, 1]);
    expect(store.getNextCursor()).toBe("cursor-2");
  });

  it("guards continuation while initial or more loading, paused, and exhausted", async () => {
    const initial = deferred<FillupPage>();
    const continuation = deferred<FillupPage>();
    vi.mocked(api.fetchFillups)
      .mockReturnValueOnce(initial.promise)
      .mockReturnValueOnce(continuation.promise);

    const firstLoad = store.setActiveVehicle(10);
    await store.loadMoreFillups();
    expect(api.fetchFillups).toHaveBeenCalledTimes(1);

    initial.resolve(page([fillup(2)], "cursor-1"));
    await firstLoad;
    const firstContinuation = store.loadMoreFillups();
    await store.loadMoreFillups();
    expect(api.fetchFillups).toHaveBeenCalledTimes(2);

    continuation.reject(new Error("offline"));
    await firstContinuation;
    expect(store.getContinuationError()).toBe("Failed to load fill-ups");
    await store.loadMoreFillups();
    expect(api.fetchFillups).toHaveBeenCalledTimes(2);

    vi.mocked(api.fetchFillups).mockResolvedValueOnce(page([fillup(1)], null));
    await store.retryLoadMoreFillups();
    expect(api.fetchFillups).toHaveBeenLastCalledWith(10, "cursor-1");
    expect(store.getContinuationError()).toBeNull();
    expect(store.getNextCursor()).toBeNull();
    await store.loadMoreFillups();
    expect(api.fetchFillups).toHaveBeenCalledTimes(3);
  });

  it("clears a previous action error before loading a continuation", async () => {
    const continuation = deferred<FillupPage>();
    vi.mocked(api.fetchFillups)
      .mockResolvedValueOnce(page([fillup(2)], "cursor-1"))
      .mockReturnValueOnce(continuation.promise);
    await store.setActiveVehicle(10);

    vi.mocked(api.deleteFillup).mockRejectedValueOnce(new Error("delete"));
    await store.deleteFillup(10, 2);
    expect(store.getError()).toBe("Failed to delete fill-up");

    const loadMore = store.loadMoreFillups();
    expect(store.getError()).toBeNull();

    continuation.resolve(page([fillup(1)], null));
    await loadMore;
  });

  it("keeps failed continuation entries and retries the unchanged cursor", async () => {
    vi.mocked(api.fetchFillups)
      .mockResolvedValueOnce(page([fillup(2)], "retry-cursor"))
      .mockRejectedValueOnce(
        new ApiError(500, "INTERNAL_ERROR", "Database error"),
      )
      .mockResolvedValueOnce(page([fillup(1)], null));
    await store.setActiveVehicle(10);

    await store.loadMoreFillups();
    expect(store.getFillups()).toEqual([fillup(2)]);
    expect(store.getNextCursor()).toBe("retry-cursor");
    expect(store.getContinuationError()).toBe("An unexpected error occurred.");

    await store.retryLoadMoreFillups();
    expect(api.fetchFillups).toHaveBeenLastCalledWith(10, "retry-cursor");
    expect(store.getFillups().map(({ id }) => id)).toEqual([2, 1]);
  });

  it("ignores an older first-page response after a fresh generation begins", async () => {
    const older = deferred<FillupPage>();
    const newer = deferred<FillupPage>();
    vi.mocked(api.fetchFillups)
      .mockReturnValueOnce(older.promise)
      .mockReturnValueOnce(newer.promise);

    const oldLoad = store.loadFillups(10);
    const freshLoad = store.loadFillups(10);
    newer.resolve(page([fillup(2)], null));
    await freshLoad;
    older.resolve(page([fillup(1)], "stale"));
    await oldLoad;

    expect(store.getFillupsByVehicle(10)).toEqual([fillup(2)]);
    expect(store.getFillupPageChain(10).generation).toBe(2);
  });

  it("ignores a stale first-page response after clearing the cache and reloading the same vehicle", async () => {
    const stale = deferred<FillupPage>();
    const fresh = deferred<FillupPage>();
    vi.mocked(api.fetchFillups)
      .mockReturnValueOnce(stale.promise)
      .mockReturnValueOnce(fresh.promise);

    const staleLoad = store.setActiveVehicle(10);
    store.clearCache();
    const freshLoad = store.setActiveVehicle(10);
    fresh.resolve(page([fillup(2)], null));
    await freshLoad;
    stale.resolve(page([fillup(1)], "stale"));
    await staleLoad;

    expect(store.getFillups()).toEqual([fillup(2)]);
    expect(store.getFillupPageChain(10).generation).toBe(2);
  });

  it("ignores a stale continuation response after a refresh generation", async () => {
    const continuation = deferred<FillupPage>();
    const fresh = deferred<FillupPage>();
    vi.mocked(api.fetchFillups)
      .mockResolvedValueOnce(page([fillup(3)], "cursor-1"))
      .mockReturnValueOnce(continuation.promise)
      .mockReturnValueOnce(fresh.promise);
    await store.setActiveVehicle(10);

    const oldContinuation = store.loadMoreFillups();
    const refresh = store.loadFillups(10);
    fresh.resolve(page([fillup(4)], null));
    await refresh;
    continuation.resolve(page([fillup(2)], "stale-cursor"));
    await oldContinuation;

    expect(store.getFillupsByVehicle(10)).toEqual([fillup(4)]);
    expect(store.getNextCursor()).toBeNull();
    expect(store.getLoadingMore()).toBe(false);
  });

  it("invalidates an old cursor when a fresh initial load fails", async () => {
    vi.mocked(api.fetchFillups)
      .mockResolvedValueOnce(page([fillup(2)], "old-cursor"))
      .mockRejectedValueOnce(new Error("offline"));
    await store.setActiveVehicle(10);

    await store.loadFillups(10);
    expect(store.getFillups()).toEqual([fillup(2)]);
    expect(store.getNextCursor()).toBeNull();

    await store.loadMoreFillups();
    expect(api.fetchFillups).toHaveBeenCalledTimes(2);
  });

  it("does not retry a continuation unless it previously failed", async () => {
    vi.mocked(api.fetchFillups).mockResolvedValue(
      page([fillup(2)], "cursor-1"),
    );
    await store.setActiveVehicle(10);

    await store.retryLoadMoreFillups();

    expect(api.fetchFillups).toHaveBeenCalledTimes(1);
    expect(store.getNextCursor()).toBe("cursor-1");
  });

  it("does not fetch continuation when form helpers inspect loaded entries", async () => {
    vi.mocked(api.fetchFillups).mockResolvedValue(
      page([fillup(2), fillup(1)], "cursor-1"),
    );
    await store.setActiveVehicle(10);

    expect(store.getFillupsByVehicle(10)[0]?.odometer).toBe(10_002);
    expect(store.getFillupsByVehicle(10)).toHaveLength(2);
    expect(api.fetchFillups).toHaveBeenCalledTimes(1);
  });

  it("retains the local create update and starts a fresh first-page generation", async () => {
    const refresh = deferred<FillupPage>();
    vi.mocked(api.fetchFillups)
      .mockResolvedValueOnce(page([fillup(1)], "old-cursor"))
      .mockReturnValueOnce(refresh.promise);
    await store.setActiveVehicle(10);
    vi.mocked(api.createFillup).mockResolvedValue(fillup(2));

    await expect(
      store.createFillup(10, {
        date: fillup(2).date,
        odometer: fillup(2).odometer,
        fuel_amount: fillup(2).fuel_amount,
        cost: fillup(2).cost,
      }),
    ).resolves.toEqual(fillup(2));
    expect(store.getFillups().map(({ id }) => id)).toEqual([2, 1]);
    expect(store.getFillupPageChain(10).generation).toBe(2);
    expect(api.fetchFillups).toHaveBeenLastCalledWith(10);

    refresh.resolve(page([fillup(3)], null));
    await vi.waitFor(() => expect(store.getFillups()).toEqual([fillup(3)]));
  });

  it("retains the local update and starts a fresh first-page generation", async () => {
    const refresh = deferred<FillupPage>();
    vi.mocked(api.fetchFillups)
      .mockResolvedValueOnce(page([fillup(2), fillup(1)], "old-cursor"))
      .mockReturnValueOnce(refresh.promise);
    await store.setActiveVehicle(10);
    const updated = fillup(1, { cost: 99 });
    vi.mocked(api.updateFillup).mockResolvedValue(updated);

    await expect(
      store.updateFillup(10, 1, {
        date: updated.date,
        odometer: updated.odometer,
        fuel_amount: updated.fuel_amount,
        cost: updated.cost,
      }),
    ).resolves.toEqual(updated);
    expect(store.getFillups().find(({ id }) => id === 1)?.cost).toBe(99);
    expect(store.getFillupPageChain(10).generation).toBe(2);

    refresh.resolve(page([fillup(3)], null));
    await vi.waitFor(() => expect(store.getFillups()).toEqual([fillup(3)]));
  });

  it("retains the local delete update and starts a fresh first-page generation", async () => {
    const refresh = deferred<FillupPage>();
    vi.mocked(api.fetchFillups)
      .mockResolvedValueOnce(page([fillup(2), fillup(1)], "old-cursor"))
      .mockReturnValueOnce(refresh.promise);
    await store.setActiveVehicle(10);
    vi.mocked(api.deleteFillup).mockResolvedValue(undefined);

    await expect(store.deleteFillup(10, 1)).resolves.toBe(true);
    expect(store.getFillups().map(({ id }) => id)).toEqual([2]);
    expect(store.getFillupPageChain(10).generation).toBe(2);

    refresh.resolve(page([fillup(3)], null));
    await vi.waitFor(() => expect(store.getFillups()).toEqual([fillup(3)]));
  });

  it("scopes mutation errors to their vehicle and preserves cached fill-ups", async () => {
    const cachedFillups = [fillup(2), fillup(1)];
    vi.mocked(api.fetchFillups).mockResolvedValue(page(cachedFillups, null));
    await store.setActiveVehicle(10);

    vi.mocked(api.createFillup).mockRejectedValueOnce(new Error("create"));
    await expect(
      store.createFillup(20, {
        date: fillup(3).date,
        odometer: fillup(3).odometer,
        fuel_amount: fillup(3).fuel_amount,
        cost: fillup(3).cost,
      }),
    ).resolves.toBeNull();
    expect(store.getError()).toBeNull();
    expect(store.getFillupsByVehicle(20)).toEqual([]);
    expect(store.getFillups()).toEqual(cachedFillups);

    vi.mocked(api.updateFillup).mockRejectedValueOnce(new Error("update"));
    await expect(
      store.updateFillup(10, 1, {
        date: fillup(1).date,
        odometer: fillup(1).odometer,
        fuel_amount: fillup(1).fuel_amount,
        cost: fillup(1).cost,
      }),
    ).resolves.toBeNull();
    expect(store.getError()).toBe("Failed to update fill-up");
    expect(store.getFillups()).toEqual(cachedFillups);

    vi.mocked(api.deleteFillup).mockRejectedValueOnce(new Error("delete"));
    await expect(store.deleteFillup(10, 1)).resolves.toBe(false);
    expect(store.getError()).toBe("Failed to delete fill-up");
    expect(store.getFillups()).toEqual(cachedFillups);
    expect(api.fetchFillups).toHaveBeenCalledTimes(1);
  });

  it("preserves initial load errors and clears them before a new action", async () => {
    vi.mocked(api.fetchFillups)
      .mockRejectedValueOnce(new Error("Network"))
      .mockResolvedValueOnce(page([fillup(1)], null));

    await store.loadFillups(10);
    expect(store.getError()).toBe("Failed to load fill-ups");
    await store.loadFillups(10);
    expect(store.getError()).toBeNull();
    expect(store.getFillupsByVehicle(10)).toEqual([fillup(1)]);
  });
});
