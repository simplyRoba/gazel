import en from "./en.json";
import de from "./de.json";
import { getSettings } from "$lib/stores/settings.svelte";

// ── Types ────────────────────────────────────────────────

type TranslationMap = Record<string, string>;

const translations: Record<string, TranslationMap> = { en, de };

export const supportedLocales = ["en", "de"] as const;
export type Locale = (typeof supportedLocales)[number];

// ── Translation function ─────────────────────────────────

/**
 * Look up a translation key for the active locale.
 * Supports `{param}` interpolation.
 * Falls back to English, then to the key itself.
 */
export function t(
  key: string,
  params?: Record<string, string | number>,
): string {
  const locale = getSettings().locale;
  let value = translations[locale]?.[key] ?? translations.en[key] ?? key;

  if (params) {
    for (const [k, v] of Object.entries(params)) {
      value = value.replace(`{${k}}`, String(v));
    }
  }

  return value;
}

/**
 * Look up a translation key for a specific locale (non-reactive, for tests).
 */
export function tWithLocale(
  locale: string,
  key: string,
  params?: Record<string, string | number>,
): string {
  let value = translations[locale]?.[key] ?? translations.en[key] ?? key;

  if (params) {
    for (const [k, v] of Object.entries(params)) {
      value = value.replace(`{${k}}`, String(v));
    }
  }

  return value;
}

// ── Exported for tests ───────────────────────────────────

export { translations };
