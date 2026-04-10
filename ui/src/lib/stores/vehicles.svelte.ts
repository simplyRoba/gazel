import type { Vehicle, CreateVehicle } from "$lib/api";
import * as api from "$lib/api";
import { t } from "$lib/i18n";
import { resolveError } from "$lib/i18n/errors";
import { pushNotification } from "$lib/stores/notifications.svelte";

let vehicles = $state<Vehicle[]>([]);
let loading = $state(false);
let error = $state<string | null>(null);

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

export async function loadVehicles(): Promise<void> {
  error = null;
  loading = true;
  try {
    vehicles = await api.fetchVehicles();
  } catch (e) {
    setError(e, "store.vehicles.loadFailed");
  } finally {
    loading = false;
  }
}

export async function createVehicle(
  data: CreateVehicle,
): Promise<Vehicle | null> {
  error = null;
  try {
    const vehicle = await api.createVehicle(data);
    vehicles = [...vehicles, vehicle];
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
    vehicles = vehicles.map((v) => (v.id === id ? vehicle : v));
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
    vehicles = vehicles.filter((v) => v.id !== id);
    return true;
  } catch (e) {
    setError(e, "store.vehicles.deleteFailed");
    return false;
  }
}
