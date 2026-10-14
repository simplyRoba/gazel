import type { SelectedStats, StatsResponse } from "#lib/api.js";
import * as api from "#lib/api.js";
import { t } from "#lib/i18n/index.js";
import { resolveError } from "#lib/i18n/errors.js";
import { pushNotification } from "#lib/stores/notifications.svelte.js";

let response = $state<StatsResponse | null>(null);
let selectedVehicleId = $state<number | null>(null);
let loading = $state(false);
let error = $state<string | null>(null);
let requestCounter = 0;
let hasRequested = false;

export function getStats(): StatsResponse | null {
  return response;
}

export function getSelectedStats(
  vehicleId: number | null,
): SelectedStats | null {
  return vehicleId !== null &&
    vehicleId === selectedVehicleId &&
    response?.selected?.vehicle_id === vehicleId
    ? response.selected
    : null;
}

export function getRequestCounter(): number {
  return requestCounter;
}

export function getSelectedVehicleId(): number | null {
  return selectedVehicleId;
}

export function getLoading(): boolean {
  return loading;
}

export function getError(): string | null {
  return error;
}

async function requestStats(
  vehicleId: number | null,
  fallbackKey: string,
): Promise<void> {
  const requestId = ++requestCounter;
  hasRequested = true;
  selectedVehicleId = vehicleId;
  error = null;
  loading = true;
  try {
    const next = await api.fetchStats(vehicleId);
    if (requestId === requestCounter) response = next;
  } catch (e) {
    if (requestId !== requestCounter) return;
    const message =
      e instanceof api.ApiError ? resolveError(e, t) : t(fallbackKey);
    error = message;
    pushNotification({ variant: "error", message });
  } finally {
    if (requestId === requestCounter) loading = false;
  }
}

export function loadStats(vehicleId: number | null = null): Promise<void> {
  return requestStats(vehicleId, "store.stats.loadFailed");
}

// An override lets a committed deletion retarget without requesting the removed ID.
export async function refreshStats(vehicleId?: number | null): Promise<void> {
  if (!hasRequested) return;
  await requestStats(
    vehicleId === undefined ? selectedVehicleId : vehicleId,
    "store.stats.refreshFailed",
  );
}

export function resetStats(): void {
  ++requestCounter;
  response = null;
  selectedVehicleId = null;
  loading = false;
  error = null;
  hasRequested = false;
}
