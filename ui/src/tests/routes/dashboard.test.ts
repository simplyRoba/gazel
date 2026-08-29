import {
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/svelte";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import type { Fillup, FillupPage, Vehicle } from "$lib/api";

const apiSpies = vi.hoisted(() => ({
  fetchFillups: vi.fn(),
}));
const vehicleStore = vi.hoisted(() => ({
  vehicles: [] as Vehicle[],
  loadVehicles: vi.fn(() => Promise.resolve()),
}));
const statsStore = vi.hoisted(() => ({
  loadAllStats: vi.fn(),
  invalidateStats: vi.fn(),
}));

vi.mock("$lib/api", async (importOriginal) => ({
  ...(await importOriginal<typeof import("$lib/api")>()),
  fetchFillups: apiSpies.fetchFillups,
}));

vi.mock("$lib/stores/vehicles.svelte", () => ({
  loadVehicles: vehicleStore.loadVehicles,
  getVehicles: () => vehicleStore.vehicles,
  getLoading: () => false,
}));

vi.mock("$lib/stores/stats.svelte", () => ({
  getVehicleStats: () => undefined,
  getVehicleHistory: () => [],
  getLoading: () => false,
  loadAllStats: statsStore.loadAllStats,
  invalidateStats: statsStore.invalidateStats,
}));

vi.mock("$lib/stores/settings.svelte", () => ({
  getSettings: () => ({
    unit_system: "metric",
    distance_unit: "km",
    volume_unit: "l",
    currency: "USD",
    color_mode: "system",
    locale: "en",
  }),
}));

import * as fillupStore from "$lib/stores/fillups.svelte";
import Dashboard from "../../routes/+page.svelte";

interface ObserverRecord {
  callback: IntersectionObserverCallback;
  options: IntersectionObserverInit | undefined;
  observe: ReturnType<typeof vi.fn>;
  disconnect: ReturnType<typeof vi.fn>;
  instance: IntersectionObserver;
}

let observers: ObserverRecord[];
let desktopMatches: boolean;
let mediaListeners: Set<() => void>;

function vehicle(id: number, name: string): Vehicle {
  return {
    id,
    name,
    make: null,
    model: null,
    year: null,
    fuel_type: "gasoline",
    notes: null,
    created_at: "2026-01-01T00:00:00Z",
    updated_at: "2026-01-01T00:00:00Z",
  };
}

function fillup(id: number, vehicleId = 1): Fillup {
  return {
    id,
    vehicle_id: vehicleId,
    date: `2026-03-${String(id).padStart(2, "0")}`,
    odometer: 10_000 + id,
    fuel_amount: 42.3,
    fuel_unit: "l",
    cost: 78.5,
    currency: "USD",
    is_full_tank: true,
    is_missed: false,
    station: `Station ${id}`,
    notes: null,
    created_at: "2026-03-01T10:00:00Z",
    updated_at: "2026-03-01T10:00:00Z",
  };
}

function page(items: Fillup[], next_cursor: string | null): FillupPage {
  return { items, next_cursor };
}

function deferred<T>() {
  let resolve!: (value: T) => void;
  let reject!: (reason?: unknown) => void;
  const promise = new Promise<T>((resolvePromise, rejectPromise) => {
    resolve = resolvePromise;
    reject = rejectPromise;
  });
  return { promise, resolve, reject };
}

function setDesktop(matches: boolean): void {
  desktopMatches = matches;
  for (const listener of mediaListeners) listener();
}

function intersect(observer: ObserverRecord, isIntersecting = true): void {
  const target = observer.observe.mock.calls[0]?.[0] as Element;
  observer.callback(
    [{ isIntersecting, target } as IntersectionObserverEntry],
    observer.instance,
  );
}

async function renderPageWithCursor(
  options: { desktop?: boolean; nextCursor?: string | null } = {},
) {
  desktopMatches = options.desktop ?? false;
  const nextCursor =
    options.nextCursor === undefined ? "cursor-1" : options.nextCursor;
  apiSpies.fetchFillups.mockResolvedValueOnce(
    page([fillup(2), fillup(1)], nextCursor),
  );
  const result = render(Dashboard);

  await waitFor(() => expect(apiSpies.fetchFillups).toHaveBeenCalledWith(1));
  if (nextCursor !== null) {
    await screen.findByTestId("fillups-sentinel");
    await waitFor(() => expect(observers.length).toBeGreaterThan(0));
  }
  return result;
}

beforeEach(() => {
  vi.clearAllMocks();
  fillupStore.clearCache();
  vehicleStore.vehicles = [vehicle(1, "First"), vehicle(2, "Second")];
  observers = [];
  desktopMatches = false;
  mediaListeners = new Set();

  vi.stubGlobal(
    "matchMedia",
    vi.fn((query: string) => ({
      media: query,
      get matches() {
        return desktopMatches;
      },
      onchange: null,
      addEventListener: (_event: string, listener: () => void) => {
        mediaListeners.add(listener);
      },
      removeEventListener: (_event: string, listener: () => void) => {
        mediaListeners.delete(listener);
      },
      addListener: vi.fn(),
      removeListener: vi.fn(),
      dispatchEvent: vi.fn(() => true),
    })),
  );

  class MockIntersectionObserver implements IntersectionObserver {
    readonly root = null;
    readonly rootMargin = "";
    readonly scrollMargin = "";
    readonly thresholds = [0];
    observe = vi.fn();
    disconnect = vi.fn();
    unobserve = vi.fn();
    takeRecords = vi.fn(() => []);

    constructor(
      callback: IntersectionObserverCallback,
      options?: IntersectionObserverInit,
    ) {
      observers.push({
        callback,
        options,
        observe: this.observe,
        disconnect: this.disconnect,
        instance: this,
      });
    }
  }

  vi.stubGlobal("IntersectionObserver", MockIntersectionObserver);
});

afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
});

describe("dashboard fill-up endless scrolling", () => {
  it("uses the desktop fill-up column as the observer root with a forward margin", async () => {
    const { container } = await renderPageWithCursor({ desktop: true });
    const observer = observers.at(-1)!;

    expect(observer.options).toMatchObject({
      root: container.querySelector(".fillups-column"),
      rootMargin: "0px 0px 240px 0px",
    });
    expect(observer.observe).toHaveBeenCalledWith(
      screen.getByTestId("fillups-sentinel"),
    );
  });

  it("uses the viewport as the mobile observer root", async () => {
    await renderPageWithCursor({ desktop: false });

    expect(observers.at(-1)?.options?.root).toBeNull();
  });

  it("requests continuation only for an intersecting sentinel and relies on store guards", async () => {
    const continuation = deferred<FillupPage>();
    apiSpies.fetchFillups
      .mockResolvedValueOnce(page([fillup(2), fillup(1)], "cursor-1"))
      .mockReturnValueOnce(continuation.promise);
    render(Dashboard);
    await screen.findByTestId("fillups-sentinel");
    const observer = observers.at(-1)!;

    intersect(observer, false);
    expect(apiSpies.fetchFillups).toHaveBeenCalledTimes(1);

    intersect(observer);
    intersect(observer);
    await waitFor(() =>
      expect(apiSpies.fetchFillups).toHaveBeenLastCalledWith(1, "cursor-1"),
    );
    expect(apiSpies.fetchFillups).toHaveBeenCalledTimes(2);

    continuation.resolve(page([fillup(0)], null));
    await waitFor(() =>
      expect(screen.queryByTestId("fillups-sentinel")).toBeNull(),
    );
  });

  it("disconnects and recreates observers for vehicle, root, and lifecycle changes", async () => {
    apiSpies.fetchFillups
      .mockResolvedValueOnce(page([fillup(2)], "first-cursor"))
      .mockResolvedValueOnce(page([fillup(4, 2)], "second-cursor"));
    const rendered = render(Dashboard);
    await screen.findByTestId("fillups-sentinel");
    const firstObserver = observers.at(-1)!;

    await fireEvent.click(screen.getByRole("button", { name: "Second" }));
    await waitFor(() =>
      expect(apiSpies.fetchFillups).toHaveBeenLastCalledWith(2),
    );
    await waitFor(() =>
      expect(firstObserver.disconnect).toHaveBeenCalledOnce(),
    );
    await waitFor(() => expect(observers.length).toBeGreaterThan(1));
    const vehicleObserver = observers.at(-1)!;

    setDesktop(true);
    await waitFor(() =>
      expect(vehicleObserver.disconnect).toHaveBeenCalledOnce(),
    );
    await waitFor(() =>
      expect(observers.at(-1)?.options?.root).toBe(
        rendered.container.querySelector(".fillups-column"),
      ),
    );
    const desktopObserver = observers.at(-1)!;

    rendered.unmount();
    expect(desktopObserver.disconnect).toHaveBeenCalledOnce();

    intersect(firstObserver);
    expect(apiSpies.fetchFillups).toHaveBeenCalledTimes(2);
  });

  it("keeps loaded cards visible and shows status while loading more", async () => {
    const continuation = deferred<FillupPage>();
    apiSpies.fetchFillups
      .mockResolvedValueOnce(page([fillup(2), fillup(1)], "cursor-1"))
      .mockReturnValueOnce(continuation.promise);
    const { container } = render(Dashboard);
    await screen.findByTestId("fillups-sentinel");

    intersect(observers.at(-1)!);

    expect(container.querySelectorAll(".fillup-card")).toHaveLength(2);
    expect((await screen.findByRole("status")).textContent).toContain(
      "Loading older fill-ups...",
    );
    continuation.resolve(page([fillup(0)], null));
    await waitFor(() =>
      expect(screen.queryByTestId("fillups-sentinel")).toBeNull(),
    );
  });

  it("keeps cards after continuation failure and retries the same cursor explicitly", async () => {
    apiSpies.fetchFillups
      .mockResolvedValueOnce(page([fillup(2), fillup(1)], "retry-cursor"))
      .mockRejectedValueOnce(new Error("offline"))
      .mockResolvedValueOnce(page([fillup(0)], null));
    const { container } = render(Dashboard);
    await screen.findByTestId("fillups-sentinel");

    intersect(observers.at(-1)!);
    const retry = await screen.findByRole("button", { name: "Try again" });

    expect(container.querySelectorAll(".fillup-card")).toHaveLength(2);
    expect(screen.getByText("Failed to load fill-ups")).toBeTruthy();
    expect(screen.queryByTestId("fillups-sentinel")).toBeNull();

    await fireEvent.click(retry);
    await waitFor(() =>
      expect(apiSpies.fetchFillups).toHaveBeenLastCalledWith(1, "retry-cursor"),
    );
    await waitFor(() =>
      expect(screen.queryByRole("button", { name: "Try again" })).toBeNull(),
    );
  });

  it("removes the exhausted sentinel and sends no requests from stale callbacks", async () => {
    apiSpies.fetchFillups
      .mockResolvedValueOnce(page([fillup(2)], "last-cursor"))
      .mockResolvedValueOnce(page([fillup(1)], null));
    render(Dashboard);
    await screen.findByTestId("fillups-sentinel");
    const observer = observers.at(-1)!;

    intersect(observer);
    await waitFor(() =>
      expect(screen.queryByTestId("fillups-sentinel")).toBeNull(),
    );
    expect(observer.disconnect).toHaveBeenCalledOnce();
    expect(apiSpies.fetchFillups).toHaveBeenCalledTimes(2);

    intersect(observer);
    expect(apiSpies.fetchFillups).toHaveBeenCalledTimes(2);
  });
});
