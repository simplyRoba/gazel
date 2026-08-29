import { SvelteMap } from "svelte/reactivity";

import type { CreateFillup, Fillup, UpdateFillup } from "$lib/api";
import * as api from "$lib/api";
import { t } from "$lib/i18n";
import { resolveError } from "$lib/i18n/errors";
import { pushNotification } from "$lib/stores/notifications.svelte";

export interface FillupPageChain {
  items: Fillup[];
  nextCursor: string | null;
  initialLoading: boolean;
  loadingMore: boolean;
  continuationError: string | null;
  generation: number;
}

interface FillupChain extends FillupPageChain {
  error: string | null;
}

const fillupCache = new SvelteMap<number, FillupChain>();
let nextGeneration = 0;
let error = $state<string | null>(null);
let activeVehicleId = $state<number | null>(null);

function emptyChain(): FillupChain {
  return {
    items: [],
    nextCursor: null,
    initialLoading: false,
    loadingMore: false,
    continuationError: null,
    generation: 0,
    error: null,
  };
}

function getChain(vehicleId: number): FillupChain {
  return fillupCache.get(vehicleId) ?? emptyChain();
}

function setChain(vehicleId: number, chain: FillupChain): void {
  fillupCache.set(vehicleId, chain);
}

function invalidateInFlightRequests(vehicleId: number): void {
  const chain = fillupCache.get(vehicleId);
  if (!chain || (!chain.initialLoading && !chain.loadingMore)) return;

  setChain(vehicleId, {
    ...chain,
    initialLoading: false,
    loadingMore: false,
    generation: ++nextGeneration,
  });
}

function messageFor(e: unknown, fallbackKey: string): string {
  return e instanceof api.ApiError ? resolveError(e, t) : t(fallbackKey);
}

function setActionError(
  vehicleId: number,
  e: unknown,
  fallbackKey: string,
): void {
  const message = messageFor(e, fallbackKey);
  const chain = getChain(vehicleId);
  setChain(vehicleId, { ...chain, error: message });
  error = message;
  pushNotification({ variant: "error", message });
}

function clearActionError(vehicleId: number): void {
  const chain = getChain(vehicleId);
  setChain(vehicleId, { ...chain, error: null });
  error = null;
}

function sortFillups(fillups: Fillup[]): Fillup[] {
  return [...fillups].sort(
    (a, b) => b.date.localeCompare(a.date) || b.id - a.id,
  );
}

function startInitialLoad(vehicleId: number): Promise<void> {
  const current = getChain(vehicleId);
  const generation = ++nextGeneration;
  setChain(vehicleId, {
    ...current,
    nextCursor: null,
    initialLoading: true,
    loadingMore: false,
    continuationError: null,
    error: null,
    generation,
  });
  error = null;

  return api
    .fetchFillups(vehicleId)
    .then((page) => {
      const chain = getChain(vehicleId);
      if (chain.generation !== generation) return;

      setChain(vehicleId, {
        ...chain,
        items: page.items,
        nextCursor: page.next_cursor,
        initialLoading: false,
        loadingMore: false,
        continuationError: null,
        error: null,
      });
    })
    .catch((e: unknown) => {
      const chain = getChain(vehicleId);
      if (chain.generation !== generation) return;

      const message = messageFor(e, "store.fillups.loadFailed");
      setChain(vehicleId, { ...chain, initialLoading: false, error: message });
      error = message;
      pushNotification({ variant: "error", message });
    });
}

// ── Getters ──────────────────────────────────────────────

export function getFillups(): Fillup[] {
  if (activeVehicleId === null) return [];
  return getFillupsByVehicle(activeVehicleId);
}

export function getFillupsByVehicle(vehicleId: number): Fillup[] {
  return fillupCache.get(vehicleId)?.items ?? [];
}

export function getLoading(): boolean {
  return activeVehicleId === null
    ? false
    : getChain(activeVehicleId).initialLoading;
}

export function getLoadingMore(): boolean {
  return activeVehicleId === null
    ? false
    : getChain(activeVehicleId).loadingMore;
}

export function getError(): string | null {
  return activeVehicleId === null ? error : getChain(activeVehicleId).error;
}

export function getContinuationError(): string | null {
  return activeVehicleId === null
    ? null
    : getChain(activeVehicleId).continuationError;
}

export function getNextCursor(): string | null {
  return activeVehicleId === null ? null : getChain(activeVehicleId).nextCursor;
}

export function getFillupPageChain(vehicleId: number): FillupPageChain {
  const chain = getChain(vehicleId);
  const { error: _, ...pageChain } = chain;
  return pageChain;
}

export function getActiveVehicleId(): number | null {
  return activeVehicleId;
}

// ── Actions ──────────────────────────────────────────────

/** Starts a new first-page generation for a vehicle. */
export async function loadFillups(vehicleId: number): Promise<void> {
  await startInitialLoad(vehicleId);
}

async function continueFillups(
  vehicleId: number,
  retry: boolean,
): Promise<void> {
  const chain = getChain(vehicleId);
  if (
    chain.initialLoading ||
    chain.loadingMore ||
    chain.nextCursor === null ||
    (retry
      ? chain.continuationError === null
      : chain.continuationError !== null)
  ) {
    return;
  }

  const { generation, nextCursor: cursor } = chain;
  setChain(vehicleId, {
    ...chain,
    loadingMore: true,
    continuationError: null,
  });

  try {
    const page = await api.fetchFillups(vehicleId, cursor);
    const current = getChain(vehicleId);
    if (current.generation !== generation) return;

    const seenIds = new Set(current.items.map((fillup) => fillup.id));
    const items = [
      ...current.items,
      ...page.items.filter((fillup) => !seenIds.has(fillup.id)),
    ];
    setChain(vehicleId, {
      ...current,
      items,
      nextCursor: page.next_cursor,
      loadingMore: false,
      continuationError: null,
    });
  } catch (e) {
    const current = getChain(vehicleId);
    if (current.generation !== generation) return;

    const message = messageFor(e, "store.fillups.loadFailed");
    setChain(vehicleId, {
      ...current,
      loadingMore: false,
      continuationError: message,
    });
    pushNotification({ variant: "error", message });
  }
}

/** Loads the next page for a vehicle when its chain can continue. */
export async function loadMoreFillups(
  vehicleId: number | null = activeVehicleId,
): Promise<void> {
  if (vehicleId === null) return;
  await continueFillups(vehicleId, false);
}

/** Retries a failed continuation with the same server cursor. */
export async function retryLoadMoreFillups(
  vehicleId: number | null = activeVehicleId,
): Promise<void> {
  if (vehicleId === null) return;
  await continueFillups(vehicleId, true);
}

export async function createFillup(
  vehicleId: number,
  data: CreateFillup,
): Promise<Fillup | null> {
  clearActionError(vehicleId);
  try {
    const fillup = await api.createFillup(vehicleId, data);
    const chain = getChain(vehicleId);
    setChain(vehicleId, {
      ...chain,
      items: sortFillups([fillup, ...chain.items]),
    });
    void startInitialLoad(vehicleId);
    return fillup;
  } catch (e) {
    setActionError(vehicleId, e, "store.fillups.createFailed");
    return null;
  }
}

export async function updateFillup(
  vehicleId: number,
  fillupId: number,
  data: UpdateFillup,
): Promise<Fillup | null> {
  clearActionError(vehicleId);
  try {
    const fillup = await api.updateFillup(vehicleId, fillupId, data);
    const chain = getChain(vehicleId);
    setChain(vehicleId, {
      ...chain,
      items: sortFillups(
        chain.items.map((existing) =>
          existing.id === fillupId ? fillup : existing,
        ),
      ),
    });
    void startInitialLoad(vehicleId);
    return fillup;
  } catch (e) {
    setActionError(vehicleId, e, "store.fillups.updateFailed");
    return null;
  }
}

export async function deleteFillup(
  vehicleId: number,
  fillupId: number,
): Promise<boolean> {
  clearActionError(vehicleId);
  try {
    await api.deleteFillup(vehicleId, fillupId);
    const chain = getChain(vehicleId);
    setChain(vehicleId, {
      ...chain,
      items: chain.items.filter((fillup) => fillup.id !== fillupId),
    });
    void startInitialLoad(vehicleId);
    return true;
  } catch (e) {
    setActionError(vehicleId, e, "store.fillups.deleteFailed");
    return false;
  }
}

export function clearCache(): void {
  fillupCache.clear();
  error = null;
  activeVehicleId = null;
}

export async function setActiveVehicle(vehicleId: number): Promise<void> {
  if (activeVehicleId !== null && activeVehicleId !== vehicleId) {
    invalidateInFlightRequests(activeVehicleId);
  }
  activeVehicleId = vehicleId;
  await loadFillups(vehicleId);
}
