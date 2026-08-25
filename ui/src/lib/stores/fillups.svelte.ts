import { SvelteMap } from "svelte/reactivity";

import type { Fillup, CreateFillup, UpdateFillup } from "$lib/api";
import * as api from "$lib/api";
import { t } from "$lib/i18n";
import { resolveError } from "$lib/i18n/errors";
import { pushNotification } from "$lib/stores/notifications.svelte";

// ── State ────────────────────────────────────────────────

const fillupCache = new SvelteMap<number, Fillup[]>();
let loading = $state(false);
let error = $state<string | null>(null);
let activeVehicleId = $state<number | null>(null);

// ── Helpers ──────────────────────────────────────────────

function setError(e: unknown, fallbackKey: string): void {
  const msg = e instanceof api.ApiError ? resolveError(e, t) : t(fallbackKey);
  error = msg;
  pushNotification({ variant: "error", message: msg });
}

// ── Getters ──────────────────────────────────────────────

export function getFillups(): Fillup[] {
  if (activeVehicleId === null) return [];
  return fillupCache.get(activeVehicleId) ?? [];
}

export function getFillupsByVehicle(vehicleId: number): Fillup[] {
  return fillupCache.get(vehicleId) ?? [];
}

export function getLoading(): boolean {
  return loading;
}

export function getError(): string | null {
  return error;
}

export function getActiveVehicleId(): number | null {
  return activeVehicleId;
}

// ── Actions ──────────────────────────────────────────────

export async function loadFillups(vehicleId: number): Promise<void> {
  error = null;
  loading = true;
  try {
    const fillups = await api.fetchFillups(vehicleId);
    fillupCache.set(vehicleId, fillups);
  } catch (e) {
    setError(e, "store.fillups.loadFailed");
  } finally {
    loading = false;
  }
}

export async function createFillup(
  vehicleId: number,
  data: CreateFillup,
): Promise<Fillup | null> {
  error = null;
  try {
    const fillup = await api.createFillup(vehicleId, data);
    const existing = fillupCache.get(vehicleId) ?? [];
    // Insert in sort order (date descending) -- newest first
    const updated = [fillup, ...existing].sort(
      (a, b) => b.date.localeCompare(a.date) || b.id - a.id,
    );
    fillupCache.set(vehicleId, updated);
    return fillup;
  } catch (e) {
    setError(e, "store.fillups.createFailed");
    return null;
  }
}

export async function updateFillup(
  vehicleId: number,
  fillupId: number,
  data: UpdateFillup,
): Promise<Fillup | null> {
  error = null;
  try {
    const fillup = await api.updateFillup(vehicleId, fillupId, data);
    const existing = fillupCache.get(vehicleId) ?? [];
    const updated = existing
      .map((f) => (f.id === fillupId ? fillup : f))
      .sort((a, b) => b.date.localeCompare(a.date) || b.id - a.id);
    fillupCache.set(vehicleId, updated);
    return fillup;
  } catch (e) {
    setError(e, "store.fillups.updateFailed");
    return null;
  }
}

export async function deleteFillup(
  vehicleId: number,
  fillupId: number,
): Promise<boolean> {
  error = null;
  try {
    await api.deleteFillup(vehicleId, fillupId);
    const existing = fillupCache.get(vehicleId) ?? [];
    fillupCache.set(
      vehicleId,
      existing.filter((f) => f.id !== fillupId),
    );
    return true;
  } catch (e) {
    setError(e, "store.fillups.deleteFailed");
    return false;
  }
}

export function clearCache(): void {
  fillupCache.clear();
  activeVehicleId = null;
}

export async function setActiveVehicle(vehicleId: number): Promise<void> {
  activeVehicleId = vehicleId;
  await loadFillups(vehicleId);
}
