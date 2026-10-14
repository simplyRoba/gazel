import { describe, it, expect, beforeEach, vi } from "vitest";
import type { FillupPage, Settings } from "#lib/api.js";

const DEFAULTS: Settings = {
  unit_system: "metric",
  distance_unit: "km",
  volume_unit: "l",
  currency: "EUR",
  color_mode: "system",
  locale: "en",
};

const mockFetchSettings = vi.fn<() => Promise<Settings>>();
const mockUpdateSettings = vi.fn<() => Promise<Settings>>();
const mockFetchFillups = vi.fn<() => Promise<FillupPage>>();
const mockRefreshStats = vi.fn<() => Promise<void>>();
vi.mock("#lib/stores/stats.svelte.js", () => ({
  refreshStats: () => mockRefreshStats(),
}));

function page(id: number): FillupPage {
  return {
    items: [
      {
        id,
        vehicle_id: 1,
        date: "2026-03-01",
        odometer: 10_000,
        fuel_amount: 40,
        fuel_unit: "l",
        cost: 60,
        currency: "EUR",
        is_full_tank: true,
        is_missed: false,
        station: null,
        notes: null,
        created_at: "2026-03-01T10:00:00Z",
        updated_at: "2026-03-01T10:00:00Z",
      },
    ],
    next_cursor: "more",
    efficiency: {
      distance_unit: "km",
      volume_unit: "l",
      values: [{ fillup_id: id, efficiency: 12.34 }],
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

vi.mock("#lib/api.js", () => ({
  fetchSettings: (...args: unknown[]) => mockFetchSettings(...(args as [])),
  fetchFillups: (...args: unknown[]) => mockFetchFillups(...(args as [])),
  updateSettings: (...args: unknown[]) => mockUpdateSettings(...(args as [])),
  ApiError: class extends Error {
    status: number;
    code: string;
    constructor(status: number, code: string, message: string) {
      super(message);
      this.status = status;
      this.code = code;
    }
  },
}));

// Mock initTheme since it touches DOM/matchMedia.
vi.mock("#lib/stores/theme.svelte.js", () => ({
  initTheme: vi.fn(),
}));

// Provide matchMedia for any modules that need it.
Object.defineProperty(window, "matchMedia", {
  writable: true,
  value: vi.fn().mockImplementation((query: string) => ({
    matches: false,
    media: query,
    addEventListener: vi.fn(),
    removeEventListener: vi.fn(),
    dispatchEvent: vi.fn(),
  })),
});

describe("settings store", () => {
  beforeEach(() => {
    vi.resetModules();
    mockFetchSettings.mockReset();
    mockUpdateSettings.mockReset();
    mockFetchFillups.mockReset();
    mockRefreshStats.mockReset().mockResolvedValue(undefined);
  });

  it.each([{ distance_unit: "mi" as const }, { volume_unit: "gal" as const }])(
    "clears all cached vehicle metadata only after a successful unit update: %j",
    async (change) => {
      const settings = await import("./settings.svelte.ts");
      const fillups = await import("./fillups.svelte.ts");
      mockFetchFillups
        .mockResolvedValueOnce(page(1))
        .mockResolvedValueOnce(page(2));
      await fillups.loadFillups(1);
      await fillups.loadFillups(2);
      const update = deferred<Settings>();
      mockUpdateSettings.mockReturnValueOnce(update.promise);
      const pending = settings.updateSettingsStore(change);
      expect(mockRefreshStats).not.toHaveBeenCalled();
      expect(fillups.getFillupPageChain(1).efficiency?.values).toEqual({
        1: 12.34,
      });
      update.resolve({ ...DEFAULTS, ...change });
      expect(await pending).toBe(true);
      expect(mockRefreshStats).toHaveBeenCalledTimes(1);
      for (const id of [1, 2]) {
        expect(fillups.getFillupPageChain(id)).toMatchObject({
          items: [{ id }],
          nextCursor: "more",
          efficiency: null,
          initialLoading: false,
          loadingMore: false,
        });
      }
      expect(mockFetchFillups).toHaveBeenCalledTimes(2);
    },
  );

  it.each(["initial", "continuation"])(
    "successful unit change rejects in-flight %s successes and failures",
    async (kind) => {
      for (const outcome of ["success", "failure"]) {
        const settings = await import("./settings.svelte.ts");
        const fillups = await import("./fillups.svelte.ts");
        fillups.clearCache();
        if (kind === "continuation") {
          mockFetchFillups.mockResolvedValueOnce(page(1));
          await fillups.setActiveVehicle(1);
        }
        const response = deferred<FillupPage>();
        mockFetchFillups.mockReturnValueOnce(response.promise);
        const load =
          kind === "initial"
            ? fillups.setActiveVehicle(1)
            : fillups.loadMoreFillups();
        const before = fillups.getFillupPageChain(1);
        const volume_unit =
          settings.getSettings().volume_unit === "l" ? "gal" : "l";
        mockUpdateSettings.mockResolvedValueOnce({
          ...settings.getSettings(),
          volume_unit,
        });
        await settings.updateSettingsStore({ volume_unit });
        expect(fillups.getFillupPageChain(1)).toMatchObject({
          items: before.items,
          nextCursor: before.nextCursor,
          efficiency: null,
          initialLoading: false,
          loadingMore: false,
        });
        if (outcome === "success") response.resolve(page(2));
        else response.reject(new Error("old units"));
        await load;
        expect(fillups.getFillupPageChain(1)).toMatchObject({
          items: before.items,
          nextCursor: before.nextCursor,
          efficiency: null,
          continuationError: null,
        });
        expect(fillups.getError()).toBeNull();
      }
    },
  );

  it.each([
    { currency: "USD" },
    { locale: "de" },
    { color_mode: "dark" as const },
  ])("does not clear badges for non-unit settings: %j", async (change) => {
    const settings = await import("./settings.svelte.ts");
    const fillups = await import("./fillups.svelte.ts");
    mockFetchFillups.mockResolvedValueOnce(page(1));
    await fillups.loadFillups(1);
    const before = fillups.getFillupPageChain(1);
    mockUpdateSettings.mockResolvedValueOnce({ ...DEFAULTS, ...change });
    expect(await settings.updateSettingsStore(change)).toBe(true);
    expect(fillups.getFillupPageChain(1)).toEqual(before);
    expect(mockFetchFillups).toHaveBeenCalledTimes(1);
    expect(mockRefreshStats).toHaveBeenCalledTimes(
      "currency" in change ? 1 : 0,
    );
  });

  it("retains metadata and in-flight requests when a unit update rolls back", async () => {
    const settings = await import("./settings.svelte.ts");
    const fillups = await import("./fillups.svelte.ts");
    mockFetchFillups.mockResolvedValueOnce(page(1));
    await fillups.setActiveVehicle(1);
    const continuation = deferred<FillupPage>();
    mockFetchFillups.mockReturnValueOnce(continuation.promise);
    const load = fillups.loadMoreFillups();
    const before = fillups.getFillupPageChain(1);
    mockUpdateSettings.mockRejectedValueOnce(new Error("Server error"));
    expect(await settings.updateSettingsStore({ distance_unit: "mi" })).toBe(
      false,
    );
    expect(settings.getSettings().distance_unit).toBe("km");
    expect(mockRefreshStats).not.toHaveBeenCalled();
    expect(fillups.getFillupPageChain(1)).toEqual(before);
    continuation.resolve(page(2));
    await load;
    expect(fillups.getFillupPageChain(1).efficiency?.values).toEqual({
      1: 12.34,
      2: 12.34,
    });
  });

  it("does not refresh unchanged units or currency", async () => {
    const settings = await import("./settings.svelte.ts");
    mockUpdateSettings.mockResolvedValue({ ...DEFAULTS });
    await settings.updateSettingsStore({
      distance_unit: "km",
      volume_unit: "l",
      currency: "EUR",
    });
    expect(mockRefreshStats).not.toHaveBeenCalled();
  });

  it("initializes with fetched settings", async () => {
    const serverSettings: Settings = { ...DEFAULTS, currency: "USD" };
    mockFetchSettings.mockResolvedValue(serverSettings);

    const { initSettings, getSettings } = await import("./settings.svelte.ts");
    await initSettings();
    expect(getSettings().currency).toBe("USD");
  });

  it("falls back to defaults when API fails", async () => {
    mockFetchSettings.mockRejectedValue(new Error("Network error"));

    const { initSettings, getSettings } = await import("./settings.svelte.ts");
    await initSettings();
    expect(getSettings()).toEqual(DEFAULTS);
  });

  it("updates settings optimistically and calls API", async () => {
    mockFetchSettings.mockResolvedValue({ ...DEFAULTS });
    mockUpdateSettings.mockResolvedValue({
      ...DEFAULTS,
      color_mode: "dark",
    });

    const { initSettings, updateSettingsStore, getSettings } =
      await import("./settings.svelte.ts");
    await initSettings();

    const result = await updateSettingsStore({ color_mode: "dark" });
    expect(result).toBe(true);
    expect(getSettings().color_mode).toBe("dark");
  });

  it("reverts on update failure", async () => {
    mockFetchSettings.mockResolvedValue({ ...DEFAULTS });
    mockUpdateSettings.mockRejectedValue(new Error("Server error"));

    const { initSettings, updateSettingsStore, getSettings } =
      await import("./settings.svelte.ts");
    await initSettings();

    const result = await updateSettingsStore({ currency: "USD" });
    expect(result).toBe(false);
    expect(getSettings().currency).toBe("EUR");
    expect(mockRefreshStats).not.toHaveBeenCalled();
  });
});
