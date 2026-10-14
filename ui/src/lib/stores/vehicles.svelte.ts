import type { Vehicle, CreateVehicle } from "#lib/api.js";
import * as api from "#lib/api.js";
import { t } from "#lib/i18n/index.js";
import { resolveError } from "#lib/i18n/errors.js";
import {
  getSelectedVehicleId,
  refreshStats,
} from "#lib/stores/stats.svelte.js";
import {
  clearCache as clearFillupCache,
  getActiveVehicleId,
  setActiveVehicle,
} from "#lib/stores/fillups.svelte.js";
import { pushNotification } from "#lib/stores/notifications.svelte.js";

let vehicles = $state<Vehicle[]>([]);
let loading = $state(false);
let error = $state<string | null>(null);
let requestCounter = 0;
let latestLoad: Promise<void> | null = null;

function setError(e: unknown, fallbackKey: string): void {
  const msg = e instanceof api.ApiError ? resolveError(e, t) : t(fallbackKey);
  error = msg;
  pushNotification({ variant: "error", message: msg });
}

export function getVehicles(): Vehicle[] {
  return vehicles;
}

export function getLoading(): boolean {
  return loading;
}

export function getError(): string | null {
  return error;
}

export function loadVehicles(): Promise<void> {
  const requestId = ++requestCounter;
  error = null;
  loading = true;
  latestLoad = readVehicles(requestId);
  return latestLoad;
}

async function readVehicles(requestId: number): Promise<void> {
  try {
    const next = await api.fetchVehicles();
    if (requestId === requestCounter) vehicles = next;
  } catch (e) {
    if (requestId === requestCounter) setError(e, "store.vehicles.loadFailed");
  } finally {
    if (requestId === requestCounter) loading = false;
    // Initialization callers must see the latest completed fleet, not just
    // ignore their stale response while a newer read is still pending.
    else if (latestLoad) await latestLoad;
  }
}

function invalidateReads(): void {
  ++requestCounter;
  loading = false;
  latestLoad = null;
}

export async function createVehicle(
  data: CreateVehicle,
): Promise<Vehicle | null> {
  error = null;
  try {
    const vehicle = await api.createVehicle(data);
    invalidateReads();
    vehicles = [...vehicles, vehicle];
    void refreshStats();
    return vehicle;
  } catch (e) {
    setError(e, "store.vehicles.createFailed");
    return null;
  }
}

export async function updateVehicle(
  id: number,
  data: CreateVehicle,
): Promise<Vehicle | null> {
  error = null;
  try {
    const vehicle = await api.updateVehicle(id, data);
    invalidateReads();
    vehicles = vehicles.map((v) => (v.id === id ? vehicle : v));
    void refreshStats();
    return vehicle;
  } catch (e) {
    setError(e, "store.vehicles.updateFailed");
    return null;
  }
}

export async function deleteVehicle(id: number): Promise<boolean> {
  error = null;
  try {
    await api.deleteVehicle(id);
    invalidateReads();
    vehicles = vehicles.filter((v) => v.id !== id);
    const next = vehicles[0]?.id ?? null;
    const activeDeleted = getActiveVehicleId() === id;
    if (activeDeleted) {
      clearFillupCache();
      if (next !== null) void setActiveVehicle(next);
    }
    void refreshStats(
      activeDeleted || getSelectedVehicleId() === id ? next : undefined,
    );
    return true;
  } catch (e) {
    setError(e, "store.vehicles.deleteFailed");
    return false;
  }
}
