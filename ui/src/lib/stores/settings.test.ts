import { describe, it, expect, beforeEach, vi } from "vitest";
import type { Settings } from "$lib/api";

const DEFAULTS: Settings = {
  unit_system: "metric",
  distance_unit: "km",
  volume_unit: "l",
  currency: "USD",
  color_mode: "system",
  locale: "en",
};

const mockFetchSettings = vi.fn<() => Promise<Settings>>();
const mockUpdateSettings = vi.fn<() => Promise<Settings>>();

vi.mock("$lib/api", () => ({
  fetchSettings: (...args: unknown[]) => mockFetchSettings(...(args as [])),
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
vi.mock("$lib/stores/theme.svelte", () => ({
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
  });

  it("initializes with fetched settings", async () => {
    const serverSettings: Settings = { ...DEFAULTS, currency: "EUR" };
    mockFetchSettings.mockResolvedValue(serverSettings);

    const { initSettings, getSettings } = await import("./settings.svelte.ts");
    await initSettings();
    expect(getSettings().currency).toBe("EUR");
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

    const result = await updateSettingsStore({ currency: "EUR" });
    expect(result).toBe(false);
    expect(getSettings().currency).toBe("USD");
  });
});
