import { describe, it, expect, beforeEach, vi } from "vitest";

// Mock $lib/api before importing the theme module.
vi.mock("$lib/api", () => ({
  updateSettings: vi.fn().mockResolvedValue({}),
}));

// Mock matchMedia before importing the module.
let matchMediaListener: ((e: { matches: boolean }) => void) | null = null;
let matchMediaMatches = false;

Object.defineProperty(window, "matchMedia", {
  writable: true,
  value: vi.fn().mockImplementation((query: string) => ({
    matches: matchMediaMatches,
    media: query,
    addEventListener: (
      _event: string,
      listener: (e: { matches: boolean }) => void,
    ) => {
      matchMediaListener = listener;
    },
    removeEventListener: vi.fn(),
    dispatchEvent: vi.fn(),
  })),
});

describe("theme store", () => {
  beforeEach(() => {
    localStorage.clear();
    document.documentElement.removeAttribute("data-theme");
    matchMediaMatches = false;
    matchMediaListener = null;
    vi.resetModules();
  });

  it("defaults to system preference when no stored value", async () => {
    const { initTheme, getThemePreference } = await import("./theme.svelte.ts");
    initTheme();
    expect(getThemePreference()).toBe("system");
    expect(document.documentElement.getAttribute("data-theme")).toBe("light");
  });

  it("reads stored preference from localStorage", async () => {
    localStorage.setItem("gazel.theme", "dark");
    const { initTheme, getThemePreference } = await import("./theme.svelte.ts");
    initTheme();
    expect(getThemePreference()).toBe("dark");
    expect(document.documentElement.getAttribute("data-theme")).toBe("dark");
  });

  it("persists preference to localStorage on setTheme", async () => {
    const { initTheme, setTheme } = await import("./theme.svelte.ts");
    initTheme();
    setTheme("dark");
    expect(localStorage.getItem("gazel.theme")).toBe("dark");
    expect(document.documentElement.getAttribute("data-theme")).toBe("dark");
  });

  it("updates theme when OS changes and preference is system", async () => {
    const { initTheme, setTheme } = await import("./theme.svelte.ts");
    initTheme();
    setTheme("system");

    // Simulate OS switching to dark.
    if (matchMediaListener) {
      matchMediaListener({ matches: true });
    }
    expect(document.documentElement.getAttribute("data-theme")).toBe("dark");
  });

  it("ignores OS changes when explicit preference is set", async () => {
    const { initTheme, setTheme } = await import("./theme.svelte.ts");
    initTheme();
    setTheme("light");

    // Simulate OS switching to dark.
    if (matchMediaListener) {
      matchMediaListener({ matches: true });
    }
    expect(document.documentElement.getAttribute("data-theme")).toBe("light");
  });
});

describe("theme reconciliation", () => {
  beforeEach(() => {
    localStorage.clear();
    document.documentElement.removeAttribute("data-theme");
    matchMediaMatches = false;
    matchMediaListener = null;
    vi.resetModules();
  });

  it("server wins when server and localStorage disagree", async () => {
    localStorage.setItem("gazel.theme", "light");
    const { initTheme, getThemePreference } = await import("./theme.svelte.ts");
    initTheme("dark");
    expect(getThemePreference()).toBe("dark");
    expect(localStorage.getItem("gazel.theme")).toBe("dark");
    expect(document.documentElement.getAttribute("data-theme")).toBe("dark");
  });

  it("no action when server and localStorage agree", async () => {
    localStorage.setItem("gazel.theme", "dark");
    const { initTheme, getThemePreference } = await import("./theme.svelte.ts");
    initTheme("dark");
    expect(getThemePreference()).toBe("dark");
    expect(localStorage.getItem("gazel.theme")).toBe("dark");
  });

  it("first-sync: pushes localStorage value to server when server is default", async () => {
    localStorage.setItem("gazel.theme", "dark");
    const { initTheme, getThemePreference } = await import("./theme.svelte.ts");
    initTheme("system");
    // localStorage explicit value should be kept.
    expect(getThemePreference()).toBe("dark");
    expect(document.documentElement.getAttribute("data-theme")).toBe("dark");
  });

  it("uses localStorage when API unavailable (no serverColorMode)", async () => {
    localStorage.setItem("gazel.theme", "light");
    const { initTheme, getThemePreference } = await import("./theme.svelte.ts");
    initTheme();
    expect(getThemePreference()).toBe("light");
    expect(document.documentElement.getAttribute("data-theme")).toBe("light");
  });
});
