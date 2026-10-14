import { beforeEach, describe, expect, it, vi } from "vitest";
import type { PeriodSeries, StatsResponse } from "#lib/api.js";
import * as api from "#lib/api.js";
import * as store from "./stats.svelte";

vi.mock("#lib/api.js", async (importOriginal) => ({
  ...(await importOriginal<typeof import("#lib/api.js")>()),
  fetchStats: vi.fn(),
  updateSettings: vi.fn(),
}));
vi.mock("#lib/stores/notifications.svelte.js", () => ({
  pushNotification: vi.fn(),
}));
import { pushNotification } from "#lib/stores/notifications.svelte.js";

function snapshot(id: number | null = null, totalCost = 250): StatsResponse {
  const summary = {
    vehicle_id: id ?? 1,
    total_distance: 1500,
    total_fuel: 120,
    total_cost: totalCost,
    fill_up_count: 5,
    average_efficiency: 12.5,
    average_cost_per_distance: 0.17,
  };
  const period: PeriodSeries = {
    points: [{ period: "2026-01", value: 250 }],
    source_count: 20,
    source_range: { start: "2024-06", end: "2026-01" },
    displayed_range: { start: "2025-02", end: "2026-01" },
    omitted_older_periods: true,
  };
  const sparkline = {
    points: [
      { position: 0, value: 12 },
      { position: 0.9, value: 15 },
      { position: 1, value: 13 },
    ],
    source_count: 100,
    reduced: true,
  };
  return {
    units: { distance_unit: "km", volume_unit: "l", currency: "EUR" },
    fleet: {
      total_distance: 1500,
      total_fuel: 120,
      total_cost: totalCost,
      fill_up_count: 5,
      average_efficiency: 12.5,
      cost_per_distance: 0.17,
      cost_per_volume: 2.08,
    },
    vehicles: [summary],
    fleet_efficiency: sparkline,
    selected:
      id === null
        ? null
        : {
            vehicle_id: id,
            summary,
            charts: {
              cost: {
                monthly: period,
                yearly: { ...period, points: [{ period: "2026", value: 250 }] },
              },
              distance: { monthly: period, yearly: period },
              fuel_price: {
                points: [{ date: "2026-01-01", value: 2 }],
                source_count: 300,
                reduced: true,
                source_range: { start: "2020-01-01", end: "2026-01-01" },
                displayed_range: { start: "2020-01-01", end: "2026-01-01" },
                trend: [
                  { date: "2020-01-01", value: 1 },
                  { date: "2026-01-01", value: 2 },
                ],
              },
            },
            cost_per_distance: sparkline,
          },
  };
}

function deferred<T>() {
  let resolve!: (value: T) => void;
  let reject!: (reason: unknown) => void;
  const promise = new Promise<T>((res, rej) => {
    resolve = res;
    reject = rej;
  });
  return { promise, resolve, reject };
}

describe("coordinated statistics", () => {
  beforeEach(() => {
    vi.resetAllMocks();
    store.resetStats();
  });

  it("exposes the existing request counter for load, refresh, and reset ordering", async () => {
    const initial = store.getRequestCounter();
    await store.refreshStats();
    expect(store.getRequestCounter()).toBe(initial);
    vi.mocked(api.fetchStats).mockResolvedValue(snapshot(1));
    await store.loadStats(1);
    expect(store.getRequestCounter()).toBe(initial + 1);
    await store.refreshStats();
    expect(store.getRequestCounter()).toBe(initial + 2);
    store.resetStats();
    expect(store.getRequestCounter()).toBe(initial + 3);
  });

  it("makes one request per fleet load and selection, preserving exact wire values", async () => {
    for (const id of [null, 1, 2]) {
      const response = snapshot(id);
      vi.mocked(api.fetchStats).mockResolvedValueOnce(response);
      await store.loadStats(id);
      expect(store.getStats()).toEqual(response);
      expect(store.getSelectedVehicleId()).toBe(id);
      expect(store.getSelectedStats(id)).toEqual(response.selected);
    }
    expect(vi.mocked(api.fetchStats).mock.calls).toEqual([[null], [1], [2]]);
  });

  it("settings refresh is dormant before first load and retains response units during optimistic updates", async () => {
    const settings = await import("./settings.svelte");
    const original = { ...settings.getSettings() };
    vi.mocked(api.updateSettings).mockResolvedValueOnce({
      ...original,
      currency: "USD",
    });
    await settings.updateSettingsStore({ currency: "USD" });
    expect(api.fetchStats).not.toHaveBeenCalled();
    vi.mocked(api.fetchStats).mockResolvedValueOnce(snapshot(1));
    await store.loadStats(1);
    vi.mocked(api.fetchStats).mockClear();
    const mutation = deferred<api.Settings>();
    const refresh = deferred<StatsResponse>();
    vi.mocked(api.updateSettings).mockReturnValueOnce(mutation.promise);
    vi.mocked(api.fetchStats).mockReturnValueOnce(refresh.promise);
    const pending = settings.updateSettingsStore({ currency: "GBP" });
    expect(settings.getSettings().currency).toBe("GBP");
    expect(store.getStats()?.units.currency).toBe("EUR");
    expect(api.fetchStats).not.toHaveBeenCalled();
    mutation.resolve({ ...original, currency: "GBP" });
    await pending;
    expect(api.fetchStats).toHaveBeenCalledExactlyOnceWith(1);
    expect(store.getStats()?.units.currency).toBe("EUR");
    refresh.reject(new Error("offline"));
    await Promise.resolve();
    await Promise.resolve();
    expect(store.getStats()?.units.currency).toBe("EUR");
    vi.mocked(api.fetchStats).mockClear();
    vi.mocked(api.updateSettings).mockRejectedValueOnce(
      new Error("failed mutation"),
    );
    await settings.updateSettingsStore({ currency: "EUR" });
    expect(api.fetchStats).not.toHaveBeenCalled();
  });

  it("keeps a newer failure when an older success completes last", async () => {
    vi.mocked(api.fetchStats).mockResolvedValueOnce(snapshot(1));
    await store.loadStats(1);
    const old = deferred<StatsResponse>();
    vi.mocked(api.fetchStats)
      .mockReturnValueOnce(old.promise)
      .mockRejectedValueOnce(new Error("new failure"));
    const pending = store.loadStats(1);
    await store.loadStats(2);
    const error = store.getError();
    expect(error).not.toBeNull();
    old.resolve(snapshot(1, 999));
    await pending;
    expect(store.getStats()).toEqual(snapshot(1));
    expect(store.getError()).toBe(error);
    expect(store.getSelectedStats(1)).toBeNull();
    expect(pushNotification).toHaveBeenCalledTimes(1);
  });

  it("reset cannot be repopulated by an in-flight response", async () => {
    const old = deferred<StatsResponse>();
    vi.mocked(api.fetchStats).mockReturnValueOnce(old.promise);
    const pending = store.loadStats(1);
    store.resetStats();
    old.resolve(snapshot(1));
    await pending;
    expect(store.getStats()).toBeNull();
    expect(store.getLoading()).toBe(false);
    expect(store.getSelectedVehicleId()).toBeNull();
    await store.refreshStats();
    expect(api.fetchStats).toHaveBeenCalledTimes(1);
  });

  it("accepts an empty fleet", async () => {
    const response: StatsResponse = {
      units: snapshot().units,
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
    vi.mocked(api.fetchStats).mockResolvedValue(response);
    await store.loadStats();
    expect(store.getStats()).toEqual(response);
    expect(api.fetchStats).toHaveBeenCalledTimes(1);
  });

  it("retains the full snapshot and its units after a failed refresh, clears errors on retry", async () => {
    const response = snapshot(1);
    vi.mocked(api.fetchStats).mockResolvedValueOnce(response);
    await store.loadStats(1);
    vi.mocked(api.fetchStats).mockRejectedValueOnce(new Error("offline"));
    await store.refreshStats();
    expect(store.getStats()).toEqual(response);
    expect(store.getSelectedStats(1)).toEqual(response.selected);
    expect(store.getError()).toBe("Failed to refresh stats");
    expect(pushNotification).toHaveBeenCalledTimes(1);
    const retry = deferred<StatsResponse>();
    vi.mocked(api.fetchStats).mockReturnValueOnce(retry.promise);
    const pending = store.refreshStats();
    expect(store.getError()).toBeNull();
    expect(store.getLoading()).toBe(true);
    expect(store.getStats()?.units).toEqual(response.units);
    retry.resolve({
      ...snapshot(1, 300),
      units: { distance_unit: "mi", volume_unit: "gal", currency: "USD" },
    });
    await pending;
    expect(store.getStats()?.units.currency).toBe("USD");
    expect(store.getLoading()).toBe(false);
    expect(vi.mocked(api.fetchStats).mock.calls).toEqual([[1], [1], [1]]);
  });

  it.each(["success", "failure"])(
    "ignores superseded %s including its finally while a newer load is pending",
    async (outcome) => {
      const old = deferred<StatsResponse>();
      const next = deferred<StatsResponse>();
      vi.mocked(api.fetchStats)
        .mockReturnValueOnce(old.promise)
        .mockReturnValueOnce(next.promise);
      const first = store.loadStats(1);
      const second = store.loadStats(2);
      if (outcome === "success") old.resolve(snapshot(1));
      else old.reject(new Error("old failure"));
      await first;
      expect(store.getLoading()).toBe(true);
      expect(store.getStats()).toBeNull();
      expect(store.getError()).toBeNull();
      next.resolve(snapshot(2));
      await second;
      expect(store.getSelectedStats(2)?.vehicle_id).toBe(2);
      expect(pushNotification).not.toHaveBeenCalled();
    },
  );

  it.each(["success", "failure"])(
    "ignores older %s completing after a newer result",
    async (outcome) => {
      const old = deferred<StatsResponse>();
      vi.mocked(api.fetchStats)
        .mockReturnValueOnce(old.promise)
        .mockResolvedValueOnce(snapshot(2));
      const first = store.loadStats(1);
      await store.loadStats(2);
      if (outcome === "success") old.resolve(snapshot(1));
      else old.reject(new Error("old failure"));
      await first;
      expect(store.getStats()).toEqual(snapshot(2));
      expect(store.getError()).toBeNull();
      expect(store.getLoading()).toBe(false);
      expect(pushNotification).not.toHaveBeenCalled();
    },
  );

  it("guards A to B to A selection races and hides unmatched selected charts", async () => {
    vi.mocked(api.fetchStats).mockResolvedValueOnce(snapshot(1));
    await store.loadStats(1);
    const b = deferred<StatsResponse>();
    const a = deferred<StatsResponse>();
    vi.mocked(api.fetchStats)
      .mockReturnValueOnce(b.promise)
      .mockReturnValueOnce(a.promise);
    const loadB = store.loadStats(2);
    expect(store.getSelectedStats(1)).toBeNull();
    expect(store.getSelectedStats(2)).toBeNull();
    expect(store.getStats()?.fleet).toEqual(snapshot(1).fleet);
    const loadA = store.loadStats(1);
    a.resolve(snapshot(1, 500));
    await loadA;
    b.resolve(snapshot(2));
    await loadB;
    expect(store.getSelectedStats(1)?.summary.total_cost).toBe(500);
    expect(store.getSelectedStats(2)).toBeNull();
  });

  it.each(["success", "failure"])(
    "reset rejects in-flight %s even after another load starts",
    async (outcome) => {
      const old = deferred<StatsResponse>();
      vi.mocked(api.fetchStats).mockReturnValueOnce(old.promise);
      const pending = store.loadStats(1);
      store.resetStats();
      await store.refreshStats();
      expect(api.fetchStats).toHaveBeenCalledTimes(1);
      expect(store.getStats()).toBeNull();
      expect(store.getLoading()).toBe(false);
      expect(store.getError()).toBeNull();
      expect(store.getSelectedVehicleId()).toBeNull();
      vi.mocked(api.fetchStats).mockResolvedValueOnce(snapshot(2));
      await store.loadStats(2);
      if (outcome === "success") old.resolve(snapshot(1));
      else old.reject(new Error("old"));
      await pending;
      expect(store.getStats()).toEqual(snapshot(2));
      expect(pushNotification).not.toHaveBeenCalled();
    },
  );

  it("refresh remains dormant before first load, and can retarget a deletion", async () => {
    await store.refreshStats();
    await store.refreshStats(2);
    expect(api.fetchStats).not.toHaveBeenCalled();
    vi.mocked(api.fetchStats).mockResolvedValue(snapshot());
    await store.loadStats(1);
    await store.refreshStats(2);
    await store.refreshStats(null);
    expect(vi.mocked(api.fetchStats).mock.calls).toEqual([[1], [2], [null]]);
  });
});
