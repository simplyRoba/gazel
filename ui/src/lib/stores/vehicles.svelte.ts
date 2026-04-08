import type { Vehicle, CreateVehicle } from "$lib/api";
import * as api from "$lib/api";

let vehicles = $state<Vehicle[]>([]);
let loading = $state(false);
let error = $state<string | null>(null);

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
    error = e instanceof api.ApiError ? e.message : "Failed to load vehicles";
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
    error = e instanceof api.ApiError ? e.message : "Failed to create vehicle";
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
    error = e instanceof api.ApiError ? e.message : "Failed to update vehicle";
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
    error = e instanceof api.ApiError ? e.message : "Failed to delete vehicle";
    return false;
  }
}
