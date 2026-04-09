import { SvelteMap } from "svelte/reactivity";

import type { VehicleStats, SegmentHistory } from "$lib/api";
import * as api from "$lib/api";
import { pushNotification } from "$lib/stores/notifications.svelte";

// ── State ────────────────────────────────────────────────

const statsCache = new SvelteMap<number, VehicleStats>();
const historyCache = new SvelteMap<number, SegmentHistory[]>();
let loading = $state(false);
let error = $state<string | null>(null);

// ── Helpers ──────────────────────────────────────────────

function setError(e: unknown, fallback: string): void {
  const msg = e instanceof api.ApiError ? e.message : fallback;
  error = msg;
  pushNotification({ variant: "error", message: msg });
}

// ── Getters ──────────────────────────────────────────────

export function getVehicleStats(vehicleId: number): VehicleStats | undefined {
  return statsCache.get(vehicleId);
}

export function getVehicleHistory(vehicleId: number): SegmentHistory[] {
  return historyCache.get(vehicleId) ?? [];
}

export function getLoading(): boolean {
  return loading;
}

export function getError(): string | null {
  return error;
}

// ── Actions ──────────────────────────────────────────────

export async function loadStats(vehicleId: number): Promise<void> {
  error = null;
  loading = true;
  try {
    const [stats, history] = await Promise.all([
      api.fetchVehicleStats(vehicleId),
      api.fetchVehicleStatsHistory(vehicleId),
    ]);
    statsCache.set(vehicleId, stats);
    historyCache.set(vehicleId, history);
  } catch (e) {
    setError(e, "Failed to load stats");
  } finally {
    loading = false;
  }
}

export async function loadAllStats(vehicleIds: number[]): Promise<void> {
  error = null;
  loading = true;
  try {
    await Promise.all(vehicleIds.map((id) => loadSingle(id)));
  } catch (e) {
    setError(e, "Failed to load fleet stats");
  } finally {
    loading = false;
  }
}

async function loadSingle(vehicleId: number): Promise<void> {
  const [stats, history] = await Promise.all([
    api.fetchVehicleStats(vehicleId),
    api.fetchVehicleStatsHistory(vehicleId),
  ]);
  statsCache.set(vehicleId, stats);
  historyCache.set(vehicleId, history);
}

export async function invalidateStats(vehicleId: number): Promise<void> {
  statsCache.delete(vehicleId);
  historyCache.delete(vehicleId);
  await loadSingle(vehicleId).catch((e) => {
    setError(e, "Failed to refresh stats");
  });
}
