import {
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/svelte";
import { tick } from "svelte";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type {
  FillupPage,
  PeriodModes,
  StatsResponse,
  Vehicle,
} from "#lib/api.js";

const api = vi.hoisted(() => ({
  fetchVehicles: vi.fn(),
  fetchFillups: vi.fn(),
  fetchStats: vi.fn(),
  previewImport: vi.fn(),
  importData: vi.fn(),
  fetchAppInfo: vi.fn(),
  deleteVehicle: vi.fn(),
}));
vi.mock("#lib/api.js", async (original) => ({
  ...(await original<typeof import("#lib/api.js")>()),
  ...api,
}));
vi.mock("#lib/stores/settings.svelte.js", () => ({
  getSettings: () => ({
    unit_system: "metric",
    distance_unit: "km",
    volume_unit: "l",
    currency: "USD",
    locale: "en",
    color_mode: "system",
  }),
  updateSettingsStore: vi.fn(),
}));
vi.mock("#lib/stores/theme.svelte.js", () => ({
  setTheme: vi.fn(),
  getThemePreference: () => "system",
}));
vi.mock("#lib/stores/notifications.svelte.js", () => ({
  pushNotification: vi.fn(),
}));
import * as fillups from "#lib/stores/fillups.svelte.js";
import * as stats from "#lib/stores/stats.svelte.js";
import * as vehicles from "#lib/stores/vehicles.svelte.js";
import Settings from "../../routes/settings/+page.svelte";
import Dashboard from "../../routes/+page.svelte";

function vehicle(id: number): Vehicle {
  return {
    id,
    name: `Vehicle ${id}`,
    make: null,
    model: null,
    year: null,
    fuel_type: "gasoline",
    notes: null,
    created_at: "",
    updated_at: "",
  };
}
function page(id: number, imported: boolean): FillupPage {
  const fillupId = imported ? 20 : 10;
  return {
    items: [
      {
        id: fillupId,
        vehicle_id: id,
        date: "2026-03-01",
        odometer: 1000,
        fuel_amount: 20,
        fuel_unit: "l",
        cost: 40,
        currency: "USD",
        is_full_tank: true,
        is_missed: false,
        station: imported ? "Imported station" : "Old station",
        notes: null,
        created_at: "",
        updated_at: "",
      },
    ],
    next_cursor: null,
    efficiency: {
      distance_unit: "km",
      volume_unit: "l",
      values: [{ fillup_id: fillupId, efficiency: null }],
    },
  };
}
function response(id: number | null): StatsResponse {
  const summary = {
    vehicle_id: id ?? 1,
    total_distance: 1000,
    total_fuel: 20,
    total_cost: 40,
    fill_up_count: 1,
    average_efficiency: null,
    average_cost_per_distance: 0.04,
  };
  const period = {
    points: [],
    source_count: 0,
    source_range: null,
    displayed_range: null,
    omitted_older_periods: false,
  };
  const periods: PeriodModes = { monthly: period, yearly: period };
  return {
    units: { distance_unit: "km", volume_unit: "l", currency: "USD" },
    fleet: {
      total_distance: 1000,
      total_fuel: 20,
      total_cost: 40,
      fill_up_count: 1,
      average_efficiency: null,
      cost_per_distance: 0.04,
      cost_per_volume: 2,
    },
    vehicles: id === null ? [] : [summary],
    fleet_efficiency: { points: [], source_count: 0, reduced: false },
    selected:
      id === null
        ? null
        : {
            vehicle_id: id,
            summary,
            cost_per_distance: { points: [], source_count: 0, reduced: false },
            charts: {
              cost: periods,
              distance: periods,
              fuel_price: {
                points: [{ date: "2026-03-01", value: 2 }],
                source_count: 1,
                reduced: false,
                source_range: { start: "2026-03-01", end: "2026-03-01" },
                displayed_range: { start: "2026-03-01", end: "2026-03-01" },
                trend: null,
              },
            },
          },
  };
}
function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((yes) => {
    resolve = yes;
  });
  return { promise, resolve };
}
beforeEach(() => {
  vi.resetAllMocks();
  fillups.clearCache();
  stats.resetStats();
  api.fetchVehicles.mockResolvedValue([vehicle(1)]);
  api.fetchFillups.mockImplementation(async (id) => page(id, false));
  api.fetchStats.mockImplementation(async (id) => response(id));
  api.previewImport.mockResolvedValue({
    preview: true,
    vehicles: 1,
    fillups: 1,
  });
  api.fetchAppInfo.mockRejectedValue(new Error("offline"));
  vi.stubGlobal(
    "matchMedia",
    vi.fn(() => ({
      matches: false,
      addEventListener: vi.fn(),
      removeEventListener: vi.fn(),
    })),
  );
  vi.stubGlobal(
    "ResizeObserver",
    class {
      observe = vi.fn();
      unobserve = vi.fn();
      disconnect = vi.fn();
    },
  );
  vi.spyOn(HTMLElement.prototype, "clientWidth", "get").mockReturnValue(640);
  vi.spyOn(HTMLElement.prototype, "clientHeight", "get").mockReturnValue(180);
  HTMLDialogElement.prototype.showModal = function () {
    this.open = true;
  };
  HTMLDialogElement.prototype.close = function () {
    this.open = false;
  };
});
afterEach(() => {
  cleanup();
  fillups.clearCache();
  stats.resetStats();
  vi.restoreAllMocks();
  vi.unstubAllGlobals();
});

async function mountDashboard() {
  const result = render(Dashboard);
  await waitFor(() =>
    expect(api.fetchStats).toHaveBeenCalledExactlyOnceWith(1),
  );
  await waitFor(() => expect(fillups.getActiveVehicleId()).toBe(1));
  return result;
}

describe("pending writes completing after dashboard mount", () => {
  it("initializes once after a concurrent layout fleet read supersedes the dashboard read", async () => {
    const dashboardFleet = deferred<Vehicle[]>();
    const layoutFleet = deferred<Vehicle[]>();
    api.fetchVehicles
      .mockReturnValueOnce(dashboardFleet.promise)
      .mockReturnValueOnce(layoutFleet.promise);
    render(Dashboard);
    await waitFor(() => expect(api.fetchVehicles).toHaveBeenCalledOnce());
    const layoutLoad = vehicles.loadVehicles();
    dashboardFleet.resolve([vehicle(1)]);
    await tick();
    expect(api.fetchStats).not.toHaveBeenCalled();
    expect(api.fetchFillups).not.toHaveBeenCalled();
    layoutFleet.resolve([vehicle(2)]);
    await layoutLoad;
    await waitFor(() =>
      expect(api.fetchStats).toHaveBeenCalledExactlyOnceWith(2),
    );
    expect(api.fetchFillups.mock.calls).toEqual([[2]]);
    expect(fillups.getActiveVehicleId()).toBe(2);
    expect(vehicles.getVehicles()).toEqual([vehicle(2)]);
  });

  it.each([
    { name: "different ID", fleet: [2], selected: 2 },
    { name: "reused ID", fleet: [1], selected: 1 },
    { name: "preserved selection", fleet: [2, 1], selected: 1 },
    { name: "empty fleet", fleet: [], selected: null },
  ])(
    "coordinates imported $name without intermediate or duplicate requests",
    async ({ fleet, selected }) => {
      const pending = deferred<{
        vehicles_created: number;
        fillups_created: number;
      }>();
      api.importData.mockReturnValue(pending.promise);
      const settings = render(Settings);
      const file = new File(["{}"], "fleet.json", { type: "application/json" });
      Object.defineProperty(file, "text", { value: async () => "{}" });
      await fireEvent.change(
        settings.container.querySelector('input[type="file"]')!,
        { target: { files: [file] } },
      );
      await screen.findByText("fleet.json");
      await fireEvent.click(
        settings.container.querySelector(".import-actions .btn-primary")!,
      );
      await waitFor(() => expect(api.importData).toHaveBeenCalledOnce());
      settings.unmount();
      const dashboard = await mountDashboard();
      await screen.findByText("Old station");
      api.fetchVehicles.mockResolvedValue(fleet.map(vehicle));
      api.fetchFillups.mockImplementation(async (id) => page(id, true));
      pending.resolve({
        vehicles_created: fleet.length,
        fillups_created: fleet.length,
      });
      await waitFor(() =>
        expect(api.fetchStats.mock.calls).toEqual([[1], [selected]]),
      );
      await tick();
      expect(fillups.getActiveVehicleId()).toBe(selected);
      expect(stats.getSelectedVehicleId()).toBe(selected);
      expect(api.fetchFillups.mock.calls).toEqual(
        selected === null ? [[1]] : [[1], [selected]],
      );
      expect(screen.queryByText("Old station")).toBeNull();
      if (selected !== null) {
        await screen.findByText("Imported station");
        expect(stats.getSelectedStats(selected)?.vehicle_id).toBe(selected);
        expect(
          dashboard.container.querySelectorAll(".chart-card"),
        ).toHaveLength(6);
        expect(
          dashboard.container.querySelectorAll(".line-point"),
        ).toHaveLength(2);
      } else {
        expect(fillups.getFillups()).toEqual([]);
        expect(
          dashboard.container.querySelectorAll(".chart-card"),
        ).toHaveLength(0);
      }
      expect(api.fetchStats.mock.calls).toEqual([[1], [selected]]);
    },
  );

  it.each([
    { name: "different ID", fleet: [2], selected: 2 },
    { name: "reused ID", fleet: [1], selected: 1 },
    { name: "empty fleet", fleet: [], selected: null },
  ])(
    "keeps imported $name when the initial dashboard fleet resolves last",
    async ({ fleet, selected }) => {
      const pendingImport = deferred<{
        vehicles_created: number;
        fillups_created: number;
      }>();
      api.importData.mockReturnValue(pendingImport.promise);
      const settings = render(Settings);
      await waitFor(() => expect(vehicles.getLoading()).toBe(false));
      expect(vehicles.getVehicles()).toEqual([vehicle(1)]);
      const file = new File(["{}"], "fleet.json", { type: "application/json" });
      Object.defineProperty(file, "text", { value: async () => "{}" });
      await fireEvent.change(
        settings.container.querySelector('input[type="file"]')!,
        { target: { files: [file] } },
      );
      await screen.findByText("fleet.json");
      await fireEvent.click(
        settings.container.querySelector(".import-actions .btn-primary")!,
      );
      await waitFor(() => expect(api.importData).toHaveBeenCalledOnce());
      settings.unmount();

      const oldFleet = deferred<Vehicle[]>();
      api.fetchVehicles.mockReturnValueOnce(oldFleet.promise);
      const dashboard = render(Dashboard);
      await waitFor(() => expect(api.fetchVehicles).toHaveBeenCalledTimes(2));
      expect(api.fetchStats).not.toHaveBeenCalled();
      api.fetchVehicles.mockResolvedValue(fleet.map(vehicle));
      api.fetchFillups.mockImplementation(async (id) => page(id, true));
      pendingImport.resolve({
        vehicles_created: fleet.length,
        fillups_created: fleet.length,
      });
      await waitFor(() =>
        expect(api.fetchStats).toHaveBeenCalledExactlyOnceWith(selected),
      );
      await waitFor(() => expect(fillups.getActiveVehicleId()).toBe(selected));
      expect(vehicles.getVehicles()).toEqual(fleet.map(vehicle));

      oldFleet.resolve([vehicle(1)]);
      await oldFleet.promise;
      await tick();
      await tick();
      expect(vehicles.getVehicles()).toEqual(fleet.map(vehicle));
      expect(fillups.getActiveVehicleId()).toBe(selected);
      expect(stats.getSelectedVehicleId()).toBe(selected);
      expect(api.fetchStats.mock.calls).toEqual([[selected]]);
      expect(api.fetchFillups.mock.calls).toEqual(
        selected === null ? [] : [[selected]],
      );
      expect(screen.queryByText("Old station")).toBeNull();
      if (selected !== null) {
        await screen.findByText("Imported station");
        expect(fillups.getFillups()).toEqual(page(selected, true).items);
        expect(stats.getSelectedStats(selected)?.vehicle_id).toBe(selected);
        expect(
          dashboard.container.querySelectorAll(".chart-card"),
        ).toHaveLength(6);
      } else {
        expect(fillups.getFillups()).toEqual([]);
        expect(screen.getByTestId("dashboard-empty")).toBeTruthy();
        expect(
          dashboard.container.querySelectorAll(".chart-card"),
        ).toHaveLength(0);
      }
    },
  );

  it("reloads cleared page metadata when a unit refresh finishes during initialization", async () => {
    await vehicles.loadVehicles();
    await fillups.setActiveVehicle(1);
    await stats.loadStats(1);
    api.fetchFillups.mockClear();
    api.fetchStats.mockClear();
    const pendingFleet = deferred<Vehicle[]>();
    api.fetchVehicles.mockReturnValueOnce(pendingFleet.promise);
    render(Dashboard);
    await waitFor(() => expect(vehicles.getLoading()).toBe(true));

    // Successful unit updates clear badges and request statistics before the
    // dashboard's pending vehicle initialization can reload its first page.
    fillups.clearEfficiencyMetadata();
    await stats.refreshStats();
    expect(fillups.getFillupPageChain(1).efficiency).toBeNull();
    api.fetchFillups.mockResolvedValue(page(1, true));
    pendingFleet.resolve([vehicle(1)]);

    await screen.findByText("Imported station");
    expect(fillups.getFillupPageChain(1).efficiency).not.toBeNull();
    expect(api.fetchFillups).toHaveBeenCalledExactlyOnceWith(1);
    expect(api.fetchStats).toHaveBeenCalledExactlyOnceWith(1);
  });

  it.each([true, false])(
    "retargets a pending selected vehicle deletion (remaining=%s)",
    async (remaining) => {
      const pending = deferred<void>();
      api.deleteVehicle.mockReturnValue(pending.promise);
      api.fetchVehicles.mockResolvedValue(
        remaining ? [vehicle(1), vehicle(2)] : [vehicle(1)],
      );
      api.fetchFillups.mockResolvedValue({
        items: [],
        next_cursor: null,
        efficiency: { distance_unit: "km", volume_unit: "l", values: [] },
      });
      await vehicles.loadVehicles();
      const deletion = vehicles.deleteVehicle(1);
      await mountDashboard();
      pending.resolve();
      expect(await deletion).toBe(true);
      await tick();
      const selected = remaining ? 2 : null;
      expect(fillups.getActiveVehicleId()).toBe(selected);
      expect(stats.getSelectedVehicleId()).toBe(selected);
      expect(api.fetchStats.mock.calls).toEqual([[1], [selected]]);
      expect(api.fetchFillups.mock.calls).toEqual(
        remaining ? [[1], [2]] : [[1]],
      );
    },
  );
});
