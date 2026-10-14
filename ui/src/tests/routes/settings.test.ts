import {
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/svelte";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { Vehicle } from "#lib/api.js";

const mocks = vi.hoisted(() => ({
  previewImport: vi.fn(),
  importData: vi.fn(),
  fetchAppInfo: vi.fn(),
  clearFillups: vi.fn(),
  setActiveVehicle: vi.fn(),
  activeVehicleId: null as number | null,
  resetStats: vi.fn(),
  loadStats: vi.fn(),
  loadVehicles: vi.fn(),
  vehicles: [] as Vehicle[],
  vehicleError: null as string | null,
  updateSettings: vi.fn(),
  notify: vi.fn(),
}));
vi.mock("#lib/api.js", async (original) => ({
  ...(await original<typeof import("#lib/api.js")>()),
  previewImport: mocks.previewImport,
  importData: mocks.importData,
  fetchAppInfo: mocks.fetchAppInfo,
}));
vi.mock("#lib/stores/vehicles.svelte.js", () => ({
  loadVehicles: mocks.loadVehicles,
  getVehicles: () => mocks.vehicles,
  getLoading: () => false,
  getError: () => mocks.vehicleError,
  deleteVehicle: vi.fn(),
}));
vi.mock("#lib/stores/fillups.svelte.js", () => ({
  clearCache: mocks.clearFillups,
  getActiveVehicleId: () => mocks.activeVehicleId,
  setActiveVehicle: mocks.setActiveVehicle,
}));
vi.mock("#lib/stores/stats.svelte.js", () => ({
  resetStats: mocks.resetStats,
  loadStats: mocks.loadStats,
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
  updateSettingsStore: mocks.updateSettings,
}));
vi.mock("#lib/stores/theme.svelte.js", () => ({
  setTheme: vi.fn(),
  getThemePreference: () => "system",
}));
vi.mock("#lib/stores/notifications.svelte.js", () => ({
  pushNotification: mocks.notify,
}));
import Settings from "../../routes/settings/+page.svelte";

async function preview() {
  const result = render(Settings);
  const input = result.container.querySelector('input[type="file"]')!;
  const file = new File(["{}"], "fleet.json", { type: "application/json" });
  Object.defineProperty(file, "text", { value: async () => "{}" });
  await fireEvent.change(input, { target: { files: [file] } });
  await screen.findByText("fleet.json");
  await waitFor(() => expect(mocks.loadVehicles).toHaveBeenCalledOnce());
  return result;
}
function confirm(container: HTMLElement) {
  return fireEvent.click(
    container.querySelector(".import-actions .btn-primary")!,
  );
}
beforeEach(() => {
  vi.resetAllMocks();
  mocks.vehicles = [];
  mocks.vehicleError = null;
  mocks.activeVehicleId = null;
  mocks.loadVehicles.mockResolvedValue(undefined);
  mocks.previewImport.mockResolvedValue({
    preview: true,
    vehicles: 1,
    fillups: 10,
  });
  mocks.importData.mockResolvedValue({
    vehicles_created: 1,
    fillups_created: 10,
  });
  mocks.fetchAppInfo.mockRejectedValue(new Error("offline"));
  mocks.loadStats.mockResolvedValue(undefined);
});
afterEach(cleanup);

describe("settings import statistics lifecycle", () => {
  it.each([null, 42])(
    "resets stale responses and loads the fresh imported selection %s after vehicles finish",
    async (id) => {
      const { container } = await preview();
      let finish!: () => void;
      mocks.loadVehicles.mockImplementationOnce(
        () =>
          new Promise<void>((resolve) => {
            finish = () => {
              mocks.vehicles =
                id === null ? [] : [{ id, name: "Imported" } as Vehicle];
              resolve();
            };
          }),
      );
      await confirm(container);
      await waitFor(() => expect(mocks.loadVehicles).toHaveBeenCalledTimes(2));
      expect(mocks.clearFillups).not.toHaveBeenCalled();
      expect(mocks.resetStats).not.toHaveBeenCalled();
      expect(mocks.loadStats).not.toHaveBeenCalled();
      finish();
      await waitFor(() =>
        expect(mocks.loadStats).toHaveBeenCalledExactlyOnceWith(id),
      );
      expect(mocks.clearFillups.mock.invocationCallOrder[0]).toBeLessThan(
        mocks.resetStats.mock.invocationCallOrder[0],
      );
      expect(mocks.loadVehicles.mock.invocationCallOrder[1]).toBeLessThan(
        mocks.clearFillups.mock.invocationCallOrder[0],
      );
      expect(mocks.resetStats.mock.invocationCallOrder[0]).toBeLessThan(
        mocks.loadStats.mock.invocationCallOrder[0],
      );
      if (id !== null) {
        expect(mocks.setActiveVehicle).toHaveBeenCalledExactlyOnceWith(id);
        expect(mocks.resetStats.mock.invocationCallOrder[0]).toBeLessThan(
          mocks.setActiveVehicle.mock.invocationCallOrder[0],
        );
        expect(mocks.setActiveVehicle.mock.invocationCallOrder[0]).toBeLessThan(
          mocks.loadStats.mock.invocationCallOrder[0],
        );
      } else expect(mocks.setActiveVehicle).not.toHaveBeenCalled();
    },
  );
  it("does not clear or refresh data after a failed import", async () => {
    const { container } = await preview();
    mocks.importData.mockRejectedValueOnce(new Error("offline"));
    await confirm(container);
    await waitFor(() =>
      expect(container.querySelector(".import-error")).toBeTruthy(),
    );
    expect(mocks.clearFillups).not.toHaveBeenCalled();
    expect(mocks.resetStats).not.toHaveBeenCalled();
    expect(mocks.loadStats).not.toHaveBeenCalled();
    expect(mocks.loadVehicles).toHaveBeenCalledOnce();
  });
  it("does not request stale vehicle IDs when reloading imported vehicles fails", async () => {
    const { container } = await preview();
    mocks.loadVehicles.mockImplementationOnce(async () => {
      mocks.vehicleError = "Failed to load vehicles";
    });
    await confirm(container);
    await waitFor(() => expect(mocks.loadVehicles).toHaveBeenCalledTimes(2));
    expect(mocks.resetStats).toHaveBeenCalledOnce();
    expect(mocks.loadStats).not.toHaveBeenCalled();
  });
});
