import type { Settings, UpdateSettingsRequest } from "$lib/api";
import * as api from "$lib/api";
import { initTheme } from "$lib/stores/theme.svelte";

// ── Defaults ─────────────────────────────────────────────

const DEFAULTS: Settings = {
  unit_system: "metric",
  distance_unit: "km",
  volume_unit: "l",
  currency: "USD",
  color_mode: "system",
  locale: "en",
};

// ── State ────────────────────────────────────────────────

let settings = $state<Settings>({ ...DEFAULTS });
let initialized = $state(false);

// ── Accessors ────────────────────────────────────────────

export function getSettings(): Settings {
  return settings;
}

export function isInitialized(): boolean {
  return initialized;
}

// ── Init ─────────────────────────────────────────────────

export async function initSettings(): Promise<void> {
  if (initialized) return;
  try {
    const fetched = await api.fetchSettings();
    settings = fetched;
    initTheme(fetched.color_mode);
  } catch {
    // API unavailable — keep defaults, theme stays as inline script set it.
    initTheme();
  }
  initialized = true;
}

// ── Update ───────────────────────────────────────────────

export async function updateSettingsStore(
  data: UpdateSettingsRequest,
): Promise<boolean> {
  const previous = { ...settings };
  // Optimistic update.
  settings = { ...settings, ...data };
  try {
    const updated = await api.updateSettings(data);
    settings = updated;
    return true;
  } catch {
    // Revert on failure.
    settings = previous;
    return false;
  }
}
