import type { Settings, UpdateSettingsRequest } from "$lib/api";
import * as api from "$lib/api";
import {
  exceptionDetails,
  reportClientDiagnostic,
} from "$lib/client-diagnostics";
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
let loading = $state(false);

// ── Accessors ────────────────────────────────────────────

export function getSettings(): Settings {
  return settings;
}

export function isInitialized(): boolean {
  return initialized;
}

export function getLoading(): boolean {
  return loading;
}

// ── Init ─────────────────────────────────────────────────

export async function initSettings(): Promise<void> {
  if (initialized) return;

  loading = true;
  reportClientDiagnostic({
    stage: "settings_initialization",
    outcome: "started",
    settings_loading: loading,
  });

  try {
    try {
      const fetched = await api.fetchSettings();
      settings = fetched;
      initTheme(fetched.color_mode);
    } catch (error) {
      reportClientDiagnostic({
        stage: "settings_initialization",
        outcome: "failed",
        ...exceptionDetails(error),
        settings_loading: loading,
      });
      // API unavailable — keep defaults, theme stays as inline script set it.
      initTheme();
    }
    initialized = true;
    loading = false;
    reportClientDiagnostic({
      stage: "settings_initialization",
      outcome: "succeeded",
      settings_loading: loading,
    });
  } catch (error) {
    loading = false;
    reportClientDiagnostic({
      stage: "settings_initialization",
      outcome: "failed",
      ...exceptionDetails(error),
      settings_loading: loading,
    });
    throw error;
  }
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
