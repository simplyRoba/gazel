import { describe, it, expect, beforeEach, vi } from "vitest";

// Mock matchMedia before importing the module
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
  });

  it("defaults to system preference when no stored value", async () => {
    const { initTheme, getThemePreference } = await import("./theme.svelte.ts");
    initTheme();
    expect(getThemePreference()).toBe("system");
    expect(document.documentElement.getAttribute("data-theme")).toBe("light");
  });

  it("reads stored preference from localStorage", async () => {
    localStorage.setItem("gazel.theme", "dark");
    // Re-import to get fresh module state
    vi.resetModules();
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
    vi.resetModules();
    const { initTheme, setTheme } = await import("./theme.svelte.ts");
    initTheme();
    setTheme("system");

    // Simulate OS switching to dark
    if (matchMediaListener) {
      matchMediaListener({ matches: true });
    }
    expect(document.documentElement.getAttribute("data-theme")).toBe("dark");
  });

  it("ignores OS changes when explicit preference is set", async () => {
    vi.resetModules();
    const { initTheme, setTheme } = await import("./theme.svelte.ts");
    initTheme();
    setTheme("light");

    // Simulate OS switching to dark
    if (matchMediaListener) {
      matchMediaListener({ matches: true });
    }
    // Should remain light
    expect(document.documentElement.getAttribute("data-theme")).toBe("light");
  });
});
