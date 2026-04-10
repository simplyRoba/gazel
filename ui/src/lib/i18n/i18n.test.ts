import { beforeEach, describe, expect, it, vi } from "vitest";
import en from "./en.json";
import de from "./de.json";

// ── Translation completeness ─────────────────────────────

describe("translation completeness", () => {
  const enKeys = Object.keys(en).sort();
  const deKeys = Object.keys(de).sort();

  it("en.json and de.json have the same keys", () => {
    expect(deKeys).toEqual(enKeys);
  });

  it("no key in de.json is missing from en.json", () => {
    const missingInEn = deKeys.filter((k) => !(k in en));
    expect(missingInEn).toEqual([]);
  });

  it("no key in en.json is missing from de.json", () => {
    const missingInDe = enKeys.filter((k) => !(k in de));
    expect(missingInDe).toEqual([]);
  });

  it("all values are non-empty strings", () => {
    for (const [key, value] of Object.entries(en)) {
      expect(value, `en.json key "${key}" is empty`).toBeTruthy();
    }
    for (const [key, value] of Object.entries(de)) {
      expect(value, `de.json key "${key}" is empty`).toBeTruthy();
    }
  });
});

// ── t() function ─────────────────────────────────────────

// We need to mock settings to control locale
vi.mock("$lib/stores/settings.svelte", () => {
  let locale = "en";
  return {
    getSettings: () => ({
      unit_system: "metric",
      distance_unit: "km",
      volume_unit: "l",
      currency: "USD",
      color_mode: "system",
      locale,
    }),
    // Helper to change locale in tests
    __setLocale: (l: string) => {
      locale = l;
    },
  };
});

describe("t()", () => {
  let t: typeof import("./index").t;
  let __setLocale: (l: string) => void;

  beforeEach(async () => {
    vi.resetModules();
    const settings = await import("$lib/stores/settings.svelte");
    __setLocale = (settings as unknown as { __setLocale: (l: string) => void })
      .__setLocale;
    __setLocale("en");
    const mod = await import("./index");
    t = mod.t;
  });

  it("returns English translation for known key", () => {
    expect(t("nav.dashboard")).toBe("Dashboard");
  });

  it("returns German translation when locale is de", () => {
    __setLocale("de");
    expect(t("nav.settings")).toBe("Einstellungen");
  });

  it("replaces parameters in translated string", () => {
    expect(t("dashboard.summary.costPer", { unit: "km" })).toBe("Cost per km");
  });

  it("replaces parameters in German", () => {
    __setLocale("de");
    expect(t("dashboard.summary.costPer", { unit: "km" })).toBe(
      "Kosten pro km",
    );
  });

  it("falls back to English for missing key in non-English locale", async () => {
    // All keys exist in de, but test the fallback mechanism
    // by using tWithLocale with a hypothetical locale
    const { tWithLocale } = await import("./index");
    expect(tWithLocale("fr", "nav.dashboard")).toBe("Dashboard");
  });

  it("returns key itself when missing from all locales", () => {
    expect(t("nonexistent.key")).toBe("nonexistent.key");
  });
});

// ── tWithLocale() ────────────────────────────────────────

describe("tWithLocale()", () => {
  let tWithLocale: typeof import("./index").tWithLocale;

  beforeEach(async () => {
    vi.resetModules();
    const mod = await import("./index");
    tWithLocale = mod.tWithLocale;
  });

  it("uses specified locale regardless of settings", () => {
    expect(tWithLocale("de", "nav.settings")).toBe("Einstellungen");
    expect(tWithLocale("en", "nav.settings")).toBe("Settings");
  });

  it("falls back to English for unknown locale", () => {
    expect(tWithLocale("fr", "nav.dashboard")).toBe("Dashboard");
  });
});

// ── resolveError() ───────────────────────────────────────

describe("resolveError()", () => {
  let resolveError: typeof import("./errors").resolveError;
  let t: typeof import("./index").t;

  beforeEach(async () => {
    vi.resetModules();
    const settings = await import("$lib/stores/settings.svelte");
    (settings as unknown as { __setLocale: (l: string) => void }).__setLocale(
      "en",
    );
    const i18nMod = await import("./index");
    t = i18nMod.t;
    const errorsMod = await import("./errors");
    resolveError = errorsMod.resolveError;
  });

  it("resolves known error code to translation", () => {
    const error = {
      code: "VEHICLE_NOT_FOUND",
      message: "Vehicle not found.",
      status: 404,
    };
    const result = resolveError(error as import("$lib/api").ApiError, t);
    expect(result).toBe("Vehicle not found.");
  });

  it("falls back to error.message for unknown code", () => {
    const error = {
      code: "SOME_UNKNOWN_CODE",
      message: "Some server message",
      status: 500,
    };
    const result = resolveError(error as import("$lib/api").ApiError, t);
    expect(result).toBe("Some server message");
  });

  it("resolves to German for known code when locale is de", async () => {
    const settings = await import("$lib/stores/settings.svelte");
    (settings as unknown as { __setLocale: (l: string) => void }).__setLocale(
      "de",
    );
    const error = {
      code: "VEHICLE_NOT_FOUND",
      message: "Vehicle not found.",
      status: 404,
    };
    const result = resolveError(error as import("$lib/api").ApiError, t);
    expect(result).toBe("Fahrzeug nicht gefunden.");
  });
});
