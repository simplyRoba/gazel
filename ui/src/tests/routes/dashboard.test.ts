import {
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/svelte";
import { tick } from "svelte";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type {
  Fillup,
  FillupPage,
  PeriodModes,
  StatsResponse,
  Vehicle,
} from "#lib/api.js";

const api = vi.hoisted(() => ({
  fetchFillups: vi.fn(),
  fetchStats: vi.fn(),
  createFillup: vi.fn(),
  updateFillup: vi.fn(),
  deleteFillup: vi.fn(),
}));
const vehicles = vi.hoisted(() => ({
  items: [] as Vehicle[],
  loading: false,
  load: vi.fn<() => Promise<void>>(),
}));
const preferences = vi.hoisted(() => ({
  distance_unit: "km",
  volume_unit: "l",
  currency: "USD",
  locale: "en",
}));
vi.mock("#lib/api.js", async (original) => ({
  ...(await original<typeof import("#lib/api.js")>()),
  ...api,
}));
vi.mock("#lib/stores/vehicles.svelte.js", () => ({
  loadVehicles: vehicles.load,
  getVehicles: () => vehicles.items,
  getLoading: () => vehicles.loading,
}));
vi.mock("#lib/stores/settings.svelte.js", () => ({
  getSettings: () => ({
    unit_system: "metric",
    color_mode: "system",
    ...preferences,
  }),
}));

import * as fillups from "#lib/stores/fillups.svelte.js";
import * as stats from "#lib/stores/stats.svelte.js";
import { formatEfficiency } from "#lib/format.js";
import { windowStart } from "#lib/components/charts/chart-test-helpers.js";
import Dashboard from "../../routes/+page.svelte";

function vehicle(id: number): Vehicle {
  return {
    id,
    name: `Vehicle ${id}`,
    make: null,
    model: null,
    year: null,
    fuel_type: "gasoline",
    notes: null,
    created_at: "",
    updated_at: "",
  };
}
function fillup(id: number, vehicle_id = 1): Fillup {
  return {
    id,
    vehicle_id,
    date: "2026-03-01",
    odometer: 10000,
    fuel_amount: 20,
    fuel_unit: "l",
    cost: 40,
    currency: "USD",
    is_full_tank: true,
    is_missed: false,
    station: `Station ${id}`,
    notes: null,
    created_at: "",
    updated_at: "",
  };
}
function page(
  ids: number[],
  next_cursor: string | null,
  vehicleId = 1,
): FillupPage {
  return {
    items: ids.map((id) => fillup(id, vehicleId)),
    next_cursor,
    efficiency: {
      distance_unit: "mi",
      volume_unit: "gal",
      values: ids.map((id) => ({
        fillup_id: id,
        efficiency: id % 2 ? 31 + id / 1000 : null,
      })),
    },
  };
}
function periods(count: number, value: number): PeriodModes {
  const points = Array.from({ length: count }, (_, i) => ({
    period: `${2010 + Math.floor(i / 12)}-${String((i % 12) + 1).padStart(2, "0")}`,
    value: value + i,
  }));
  const monthly = { points };
  return {
    monthly,
    yearly: {
      ...monthly,
      points: points.map((p, i) => ({
        ...p,
        period: String(2015 + i),
        value: value * 12 + i,
      })),
    },
  };
}
function response(
  id: number | null = 1,
  count = 12,
  prices = 360,
): StatsResponse {
  const summary = {
    vehicle_id: id ?? 1,
    total_distance: 123456,
    total_fuel: 9876,
    total_cost: 23456.78,
    fill_up_count: 360,
    average_efficiency: 13.37,
    average_cost_per_distance: 0.19,
  };
  const sparkline = {
    source_count: 359,
    reduced: true,
    points: Array.from({ length: 60 }, (_, i) => ({
      position: (i / 59) ** 2,
      value: 5 + (i % 7),
    })),
  };
  const fullPrices = Array.from({ length: 360 }, (_, i) => ({
    date: new Date(Date.UTC(2020, 0, 1 + i)).toISOString().slice(0, 10),
    value: 1.25 + (i % 17) / 10,
  }));
  const pricePoints = fullPrices.slice(0, prices);
  const meanX = 359 / 2;
  const meanY = fullPrices.reduce((sum, p) => sum + p.value, 0) / 360;
  const slope =
    fullPrices.reduce((sum, p, i) => sum + (i - meanX) * (p.value - meanY), 0) /
    fullPrices.reduce((sum, _, i) => sum + (i - meanX) ** 2, 0);
  return {
    units: { distance_unit: "km", volume_unit: "l", currency: "EUR" },
    fleet: {
      ...summary,
      average_efficiency: 14.25,
      cost_per_distance: 0.29,
      cost_per_volume: 2.37,
    },
    vehicles: [summary],
    fleet_efficiency: sparkline,
    selected:
      id === null
        ? null
        : {
            vehicle_id: id,
            summary,
            cost_per_distance: {
              ...sparkline,
              points: sparkline.points.map((p) => ({
                ...p,
                value: p.value / 100,
              })),
            },
            charts: {
              cost: periods(count, 100),
              distance: periods(count, 1000),
              fuel_price: {
                points: pricePoints,
                trend:
                  prices > 1
                    ? [
                        {
                          date: fullPrices[0].date,
                          value: meanY - slope * meanX,
                        },
                        {
                          date: fullPrices.at(-1)!.date,
                          value: meanY + slope * meanX,
                        },
                      ]
                    : null,
              },
            },
          },
  };
}
function longResponse(id = 1, start = 0, end = 156): StatsResponse {
  const result = response(id, end);
  for (const series of [
    result.selected!.charts.cost,
    result.selected!.charts.distance,
  ]) {
    series.monthly.points = series.monthly.points.slice(start);
    series.yearly.points = Array.from(
      { length: Math.ceil(end / 12) - Math.floor(start / 12) },
      (_, i) => ({
        period: String(2010 + Math.floor(start / 12) + i),
        value: i + 1,
      }),
    );
  }
  return result;
}
function chartCards(container: HTMLElement) {
  return Array.from(container.querySelectorAll<HTMLElement>(".chart-card"));
}
function nav(card: HTMLElement, direction: "Previous" | "Next") {
  return card.querySelector<HTMLButtonElement>(
    `button[aria-label="${direction} year"]`,
  )!;
}
function deferred<T>() {
  let resolve!: (value: T) => void;
  let reject!: (reason: unknown) => void;
  const promise = new Promise<T>((yes, no) => {
    resolve = yes;
    reject = no;
  });
  return { resolve, reject, promise };
}
interface Observer {
  callback: IntersectionObserverCallback;
  options?: IntersectionObserverInit;
  instance: IntersectionObserver;
  observe: ReturnType<typeof vi.fn>;
  disconnect: ReturnType<typeof vi.fn>;
}
let observers: Observer[];
let desktop = false;
let mediaListeners: Set<() => void>;
function intersect(observer = observers.at(-1)!, visible = true) {
  observer.callback(
    [
      {
        target: observer.observe.mock.calls[0]?.[0],
        isIntersecting: visible,
      } as IntersectionObserverEntry,
    ],
    observer.instance,
  );
}
async function mount(nextCursor: string | null = null) {
  api.fetchFillups.mockResolvedValueOnce(page([2, 1], nextCursor));
  const result = render(Dashboard);
  await waitFor(() => expect(api.fetchStats).toHaveBeenCalledTimes(1));
  await waitFor(() =>
    expect(result.container.querySelectorAll(".chart-card")).toHaveLength(6),
  );
  if (nextCursor)
    await waitFor(() => expect(observers.length).toBeGreaterThan(0));
  return result;
}
function geometry(container: HTMLElement) {
  return {
    paths: Array.from(
      container.querySelectorAll(".chart-container path, .sparkline-bg path"),
      (p) => p.getAttribute("d"),
    ),
    bars: Array.from(container.querySelectorAll(".bars rect"), (p) =>
      ["x", "y", "width", "height"].map((a) => p.getAttribute(a)),
    ),
    dots: Array.from(container.querySelectorAll(".line-point"), (p) => [
      p.getAttribute("cx"),
      p.getAttribute("cy"),
    ]),
    axes: Array.from(
      container.querySelectorAll(".axis-x, .axis-y"),
      (p) => p.textContent,
    ),
    summaries: [
      container.querySelector(".summary-grid")?.textContent,
      container.querySelector(".vehicle-stats-row")?.textContent,
    ],
  };
}
beforeEach(() => {
  vi.resetAllMocks();
  fillups.clearCache();
  stats.resetStats();
  vehicles.items = [vehicle(1), vehicle(2)];
  vehicles.loading = false;
  vehicles.load.mockResolvedValue();
  Object.assign(preferences, {
    distance_unit: "km",
    volume_unit: "l",
    currency: "USD",
    locale: "en",
  });
  api.fetchStats.mockImplementation(async (id) => response(id));
  observers = [];
  desktop = false;
  mediaListeners = new Set();
  vi.spyOn(HTMLElement.prototype, "clientWidth", "get").mockReturnValue(640);
  vi.spyOn(HTMLElement.prototype, "clientHeight", "get").mockReturnValue(180);
  vi.stubGlobal(
    "matchMedia",
    vi.fn(() => ({
      get matches() {
        return desktop;
      },
      addEventListener: (_: string, cb: () => void) => mediaListeners.add(cb),
      removeEventListener: (_: string, cb: () => void) =>
        mediaListeners.delete(cb),
    })),
  );
  vi.stubGlobal(
    "IntersectionObserver",
    class {
      observe = vi.fn();
      disconnect = vi.fn();
      constructor(
        callback: IntersectionObserverCallback,
        options?: IntersectionObserverInit,
      ) {
        observers.push({
          callback,
          options,
          instance: this as unknown as IntersectionObserver,
          observe: this.observe,
          disconnect: this.disconnect,
        });
      }
    },
  );
  vi.stubGlobal(
    "ResizeObserver",
    class {
      observe = vi.fn();
      unobserve = vi.fn();
      disconnect = vi.fn();
    },
  );
  HTMLDialogElement.prototype.showModal = function () {
    this.open = true;
  };
  HTMLDialogElement.prototype.close = function () {
    this.open = false;
  };
});
afterEach(() => {
  cleanup();
  stats.resetStats();
  vi.restoreAllMocks();
  vi.unstubAllGlobals();
});

describe("dashboard chart consumers", () => {
  it("navigates complete independent histories locally and shares windows between layouts", async () => {
    api.fetchStats.mockResolvedValue(longResponse());
    const { container } = await mount("older");
    const cards = chartCards(container);
    const priceBefore = [geometry(cards[2]), geometry(cards[5])];
    const summaries = () => ({
      fleet: screen.getByTestId("summary-cards").textContent,
      selected: screen.getByTestId("vehicle-stats").textContent,
    });
    const summariesBefore = summaries();
    expect(summariesBefore.fleet).toContain("€23,456.78");
    expect(summariesBefore.fleet).toContain(
      formatEfficiency(14.25, "km", "l", "en"),
    );
    expect(summariesBefore.selected).toContain("€23,456.78");
    expect(summariesBefore.selected).toContain(
      formatEfficiency(13.37, "km", "l", "en"),
    );
    expect(stats.getSelectedStats(1)!.charts.fuel_price.points).toHaveLength(
      360,
    );
    expect(windowStart(cards[0])).toBe("Jan 22");
    expect(windowStart(cards[3])).toBe("Jan 22");
    await fireEvent.click(nav(cards[0], "Previous"));
    expect(windowStart(cards[0])).toBe("Jan 21");
    expect(windowStart(cards[3])).toBe("Jan 21");
    expect(windowStart(cards[1])).toBe("Jan 22");
    expect(windowStart(cards[4])).toBe("Jan 22");
    expect(summaries()).toEqual(summariesBefore);
    expect([geometry(cards[2]), geometry(cards[5])]).toEqual(priceBefore);
    api.fetchFillups.mockResolvedValueOnce(page([0], null));
    intersect();
    await waitFor(() =>
      expect(container.querySelectorAll(".fillup-card")).toHaveLength(3),
    );
    expect(windowStart(cards[0])).toBe("Jan 21");
    expect(windowStart(cards[1])).toBe("Jan 22");
    expect([geometry(cards[2]), geometry(cards[5])]).toEqual(priceBefore);
    for (let i = 0; i < 11; i++)
      await fireEvent.click(nav(cards[3], "Previous"));
    expect(windowStart(cards[0])).toBe("Jan 10");
    expect(nav(cards[0], "Previous").disabled).toBe(true);
    expect(summaries()).toEqual(summariesBefore);
    await fireEvent.click(nav(cards[3], "Next"));
    expect(windowStart(cards[0])).toBe("Jan 11");
    expect(summaries()).toEqual(summariesBefore);
    await fireEvent.click(nav(cards[0], "Previous"));
    expect(windowStart(cards[0])).toBe("Jan 10");
    await fireEvent.click(nav(cards[4], "Previous"));
    expect(windowStart(cards[1])).toBe("Jan 21");
    expect(summaries()).toEqual(summariesBefore);
    const yearly = Array.from(cards[0].querySelectorAll("button")).find((b) =>
      b.textContent?.includes("Yearly"),
    )!;
    await fireEvent.click(yearly);
    expect(cards[0].querySelectorAll(".bars rect")).toHaveLength(13);
    expect(cards[0].textContent).toContain("2010");
    expect(nav(cards[0], "Previous")).toBeNull();
    expect(summaries()).toEqual(summariesBefore);
    await fireEvent.click(
      Array.from(cards[0].querySelectorAll("button")).find((b) =>
        b.textContent?.includes("Monthly"),
      )!,
    );
    expect(windowStart(cards[0])).toBe("Jan 10");
    expect(summaries()).toEqual(summariesBefore);
    expect(api.fetchStats).toHaveBeenCalledTimes(1);
    expect(api.fetchFillups).toHaveBeenCalledTimes(2);
  });

  it("follows latest on refresh, preserves calendar selection across prepends, clamps deletions and resets on vehicle change", async () => {
    api.fetchStats.mockResolvedValue(longResponse(1, 12, 144));
    const { container } = await mount();
    const cards = chartCards(container);
    api.fetchStats.mockResolvedValueOnce(longResponse(1, 12, 156));
    await stats.refreshStats();
    await tick();
    expect(windowStart(cards[0])).toBe("Jan 22");
    await fireEvent.click(nav(cards[0], "Previous"));
    expect(windowStart(cards[0])).toBe("Jan 21");
    api.fetchStats.mockResolvedValueOnce(longResponse(1, 0, 168));
    await stats.refreshStats();
    await tick();
    expect(windowStart(cards[0])).toBe("Jan 21");
    expect(windowStart(cards[1])).toBe("Jan 23");
    api.fetchStats.mockResolvedValueOnce(longResponse(1, 144, 168));
    await stats.refreshStats();
    await tick();
    expect(windowStart(cards[0])).toBe("Jan 22");
    api.fetchStats.mockResolvedValueOnce(longResponse(2, 0, 156));
    api.fetchFillups.mockResolvedValueOnce(page([3], null, 2));
    await fireEvent.click(screen.getByRole("button", { name: "Vehicle 2" }));
    await waitFor(() =>
      expect(windowStart(chartCards(container)[0])).toBe("Jan 22"),
    );
    expect(nav(chartCards(container)[0], "Next").disabled).toBe(true);
  });
  it.each([false, true])(
    "keeps both chart layouts, axes, retained tooltips, exact summaries and sparklines stable across continuation (desktop=%s)",
    async (isDesktop) => {
      desktop = isDesktop;
      const ids = Array.from({ length: 360 }, (_, i) => 360 - i);
      api.fetchFillups
        .mockResolvedValueOnce(page(ids.slice(0, 25), "page-2"))
        .mockResolvedValueOnce(page(ids.slice(25, 50), "page-3"))
        .mockResolvedValueOnce(page(ids.slice(50, 75), null));
      const { container } = render(Dashboard);
      await waitFor(() =>
        expect(container.querySelectorAll(".chart-container svg")).toHaveLength(
          6,
        ),
      );
      await waitFor(() => expect(observers.length).toBeGreaterThan(0));
      expect(container.querySelectorAll(".fillup-card")).toHaveLength(25);
      expect(container.querySelectorAll(".bars rect")).toHaveLength(24);
      expect(container.querySelectorAll(".sparkline-bg svg")).toHaveLength(2);
      expect(screen.getByTestId("summary-cards").textContent).toContain("360");
      expect(screen.getByTestId("summary-cards").textContent).toContain(
        "€23,456.78",
      );
      expect(screen.getByTestId("summary-cards").textContent).toContain(
        formatEfficiency(14.25, "km", "l", "en"),
      );
      expect(screen.getByTestId("vehicle-stats").textContent).toContain(
        formatEfficiency(13.37, "km", "l", "en"),
      );
      const before = geometry(container);
      const efficiencyPath = container
        .querySelector('.sparkline-bg path[fill="none"]')!
        .getAttribute("d")!;
      expect(efficiencyPath).toMatch(/^M0,90 C/);
      expect(efficiencyPath).toContain(
        ` ${100 * (1 / 59) ** 2},${100 - 80 / 6 - 10}`,
      );
      const trend = response().selected!.charts.fuel_price.trend!;
      const renderedTrends = container.querySelectorAll(
        'path[stroke-dasharray="6,4"]',
      );
      expect(renderedTrends).toHaveLength(2);
      for (const path of renderedTrends) {
        const coordinates = path
          .getAttribute("d")!
          .match(/-?\d+(?:\.\d+)?/g)!
          .map(Number);
        expect(coordinates[0]).toBe(0);
        expect(coordinates[2]).toBe(578);
        expect(coordinates[1]).toBeCloseTo(
          ((2.85 - trend[0].value) / (2.85 - 1)) * 140,
        );
        expect(coordinates[3]).toBeCloseTo(
          ((2.85 - trend[1].value) / (2.85 - 1)) * 140,
        );
      }
      expect(
        before.paths.every(
          (d) => d?.startsWith("M") && !/NaN|Infinity/.test(d),
        ),
      ).toBe(true);
      // Both layouts expose the actual first price.
      const tooltips = async () => {
        const values: string[] = [];
        for (const card of container.querySelectorAll(".chart-card")) {
          const overlay = card.querySelector<SVGRectElement>(
            'rect[role="presentation"]',
          );
          if (!overlay) continue;
          overlay.getBoundingClientRect = () => ({ left: 0 }) as DOMRect;
          await fireEvent.mouseMove(overlay, { clientX: 0 });
          values.push(
            card.querySelector(".chart-tooltip")?.textContent ??
              card.textContent ??
              "",
          );
        }
        return values;
      };
      const details = await tooltips();
      expect(details.filter((s) => s.includes("€1.25/L"))).toHaveLength(2);
      const badgeCounts = [
        container.querySelectorAll('[data-testid="efficiency-badge"]').length,
      ];
      for (const count of [50, 75]) {
        intersect();
        await waitFor(() =>
          expect(container.querySelectorAll(".fillup-card")).toHaveLength(
            count,
          ),
        );
        expect(geometry(container)).toEqual(before);
        expect(await tooltips()).toEqual(details);
        badgeCounts.push(
          container.querySelectorAll('[data-testid="efficiency-badge"]').length,
        );
      }
      expect(badgeCounts[1]).toBeGreaterThan(badgeCounts[0]);
      expect(badgeCounts[2]).toBeGreaterThan(badgeCounts[1]);
      expect(api.fetchStats).toHaveBeenCalledExactlyOnceWith(1);
      expect(api.fetchFillups.mock.calls).toEqual([
        [1],
        [1, "page-2"],
        [1, "page-3"],
      ]);
      // Same-day/odometer entries receive only their own ID's page metadata.
      for (const card of container.querySelectorAll(".fillup-card")) {
        const id = Number(
          card
            .querySelector(".fillup-station")!
            .textContent!.replace("Station ", ""),
        );
        expect(
          card.querySelector('[data-testid="efficiency-badge"]')?.textContent ??
            null,
        ).toBe(
          id % 2 ? formatEfficiency(31 + id / 1000, "mi", "gal", "en") : null,
        );
      }
      for (const card of container.querySelectorAll(".chart-card")) {
        const yearly = Array.from(card.querySelectorAll("button")).find((b) =>
          b.textContent?.includes("Yearly"),
        );
        if (yearly) await fireEvent.click(yearly);
      }
      expect(container.querySelectorAll(".bars rect")).toHaveLength(24);
      expect(container.querySelector(".axis-x")?.textContent).toContain("2015");
      expect(api.fetchStats).toHaveBeenCalledTimes(1);
      expect(api.fetchFillups).toHaveBeenCalledTimes(3);
    },
  );

  it.each([0, 1, 2])(
    "preserves chart thresholds with %s periods and the first eligible price",
    async (count) => {
      api.fetchStats.mockResolvedValue(response(1, count, 1));
      const { container } = await mount();
      expect(container.querySelectorAll(".chart-empty")).toHaveLength(
        count === 0 ? 4 : count === 1 ? 2 : 0,
      );
      expect(container.querySelectorAll(".line-point")).toHaveLength(2);
      for (const dot of container.querySelectorAll(".line-point"))
        expect(Number(dot.getAttribute("cx"))).toBeGreaterThan(0);
    },
  );

  it("loads once per mount and selection, including selection from the global CTA setter", async () => {
    api.fetchFillups.mockResolvedValue(page([1], null));
    const rendered = render(Dashboard);
    await waitFor(() =>
      expect(api.fetchStats).toHaveBeenCalledExactlyOnceWith(1),
    );
    await fireEvent.click(screen.getByRole("button", { name: "Vehicle 2" }));
    await waitFor(() => expect(api.fetchStats.mock.calls).toEqual([[1], [2]]));
    fillups.setActiveVehicle(1);
    await waitFor(() =>
      expect(api.fetchStats.mock.calls).toEqual([[1], [2], [1]]),
    );
    rendered.unmount();
    render(Dashboard);
    await waitFor(() =>
      expect(api.fetchStats.mock.calls).toEqual([[1], [2], [1], [1]]),
    );
  });

  it("retains matching charts with response units on failed refresh, but hides them for a new selection", async () => {
    const { container } = await mount();
    const before = geometry(container);
    Object.assign(preferences, {
      distance_unit: "mi",
      volume_unit: "gal",
      currency: "USD",
    });
    api.fetchStats.mockRejectedValueOnce(new Error("offline"));
    await stats.refreshStats();
    await screen.findByRole("alert");
    expect(geometry(container)).toEqual(before);
    expect(container.querySelectorAll(".chart-title")[2].textContent).toContain(
      "EUR/L",
    );
    const pending = deferred<StatsResponse>();
    api.fetchStats.mockReturnValueOnce(pending.promise);
    api.fetchFillups.mockResolvedValueOnce(page([3], null, 2));
    await fireEvent.click(screen.getByRole("button", { name: "Vehicle 2" }));
    expect(container.querySelectorAll(".chart-card")).toHaveLength(0);
    expect(container.querySelectorAll(".skeleton-chart")).toHaveLength(4);
    pending.reject(new Error("offline"));
    await screen.findByRole("alert");
    expect(container.querySelectorAll(".chart-container path")).toHaveLength(0);
    expect(container.querySelectorAll(".chart-card")).toHaveLength(6);
  });

  it("keeps matching charts visible throughout a pending same-selection refresh", async () => {
    const { container } = await mount();
    const before = geometry(container);
    const pending = deferred<StatsResponse>();
    api.fetchStats.mockReturnValueOnce(pending.promise);
    const refresh = stats.refreshStats();
    await tick();
    expect(screen.getByRole("status").textContent).toContain(
      "Refreshing statistics",
    );
    expect(container.querySelectorAll(".skeleton-chart")).toHaveLength(0);
    expect(geometry(container)).toEqual(before);
    pending.resolve(response());
    await refresh;
  });

  it("renders no efficiency badges when page metadata has been cleared", async () => {
    const { container } = await mount();
    expect(
      container.querySelectorAll('[data-testid="efficiency-badge"]'),
    ).toHaveLength(1);
    fillups.clearEfficiencyMetadata();
    await tick();
    expect(
      container.querySelectorAll('[data-testid="efficiency-badge"]'),
    ).toHaveLength(0);
    expect(api.fetchStats).toHaveBeenCalledTimes(1);
  });

  it("shows initial statistics skeletons in both layouts until the first response", async () => {
    const pending = deferred<StatsResponse>();
    api.fetchStats.mockReturnValueOnce(pending.promise);
    api.fetchFillups.mockResolvedValueOnce(page([], null));
    const { container } = render(Dashboard);
    await waitFor(() =>
      expect(api.fetchStats).toHaveBeenCalledExactlyOnceWith(1),
    );
    expect(container.querySelectorAll(".skeleton-chart")).toHaveLength(4);
    expect(screen.getByTestId("summary-cards-loading")).toBeTruthy();
    pending.resolve(response(1, 0, 0));
    await waitFor(() =>
      expect(container.querySelectorAll(".chart-empty")).toHaveLength(6),
    );
  });

  it("does not continue initialization after unmount", async () => {
    const pending = deferred<void>();
    vehicles.load.mockReturnValueOnce(pending.promise);
    const view = render(Dashboard);
    view.unmount();
    pending.resolve();
    await tick();
    expect(api.fetchStats).not.toHaveBeenCalled();
    expect(api.fetchFillups).not.toHaveBeenCalled();
  });

  it("keeps a valid existing active vehicle on mount", async () => {
    api.fetchFillups.mockResolvedValue(page([1], null, 2));
    fillups.setActiveVehicle(2);
    render(Dashboard);
    await waitFor(() =>
      expect(api.fetchStats).toHaveBeenCalledExactlyOnceWith(2),
    );
    expect(fillups.getActiveVehicleId()).toBe(2);
  });

  it("shows the empty fleet without charts and requests fleet statistics only once", async () => {
    vehicles.items = [];
    const { container } = render(Dashboard);
    expect(screen.getByTestId("dashboard-empty")).toBeTruthy();
    await waitFor(() =>
      expect(api.fetchStats).toHaveBeenCalledExactlyOnceWith(null),
    );
    expect(container.querySelectorAll(".chart-card")).toHaveLength(0);
    expect(api.fetchFillups).not.toHaveBeenCalled();
  });

  it("drops a deleted selection when remounting with an empty fleet", async () => {
    api.fetchFillups.mockResolvedValueOnce(page([1], null));
    await fillups.setActiveVehicle(1);
    api.fetchFillups.mockClear();
    vehicles.items = [];
    const { container } = render(Dashboard);
    await waitFor(() =>
      expect(api.fetchStats).toHaveBeenCalledExactlyOnceWith(null),
    );
    expect(fillups.getActiveVehicleId()).toBeNull();
    expect(fillups.getFillups()).toEqual([]);
    expect(container.querySelectorAll(".chart-card")).toHaveLength(0);
    expect(api.fetchFillups).not.toHaveBeenCalled();
  });

  it("mirrors dashboard structure while vehicles load", () => {
    vehicles.loading = true;
    vehicles.load.mockReturnValueOnce(deferred<void>().promise);
    const { container } = render(Dashboard);
    expect(
      screen.getByTestId("dashboard-loading").getAttribute("aria-busy"),
    ).toBe("true");
    expect(container.querySelectorAll(".skeleton-summary")).toHaveLength(4);
    expect(container.querySelectorAll(".skeleton-fillup")).toHaveLength(3);
  });
});

describe("dashboard fill-up mutation refreshes", () => {
  it.each(["create", "update", "delete"] as const)(
    "refreshes once after successful %s and preserves the first-page reload",
    async (action) => {
      api.fetchFillups.mockResolvedValue(
        page(action === "create" ? [] : [1], null),
      );
      const { container } = render(Dashboard);
      await waitFor(() => expect(api.fetchStats).toHaveBeenCalledTimes(1));
      await waitFor(() => expect(fillups.getLoading()).toBe(false));
      if (action === "create") {
        api.createFillup.mockResolvedValueOnce(fillup(2));
        await fireEvent.click(
          screen.getByRole("button", { name: "Add fill-up" }),
        );
        for (const [id, value] of [
          ["q-odometer", "11000"],
          ["q-fuel", "40"],
          ["q-total", "80"],
        ])
          await fireEvent.input(container.querySelector(`#${id}`)!, {
            target: { value },
          });
        await fireEvent.submit(container.querySelector("#quick-fill-form")!);
      } else {
        await fireEvent.click(container.querySelector(".fillup-card")!);
        if (action === "update") {
          api.updateFillup.mockResolvedValueOnce(fillup(1));
          await fireEvent.submit(container.querySelector("#fillup-form")!);
        } else {
          api.deleteFillup.mockResolvedValueOnce(undefined);
          await fireEvent.click(
            container.querySelector("#fillup-form .btn-danger")!,
          );
          await fireEvent.click(
            container.querySelector(".modal-dialog[open] .btn-danger")!,
          );
        }
      }
      await waitFor(() => expect(api.fetchStats).toHaveBeenCalledTimes(2));
      expect(api.fetchFillups.mock.calls).toEqual([[1], [1]]);
      expect(api.fetchStats.mock.calls).toEqual([[1], [1]]);
    },
  );

  it.each(["create", "update", "delete"] as const)(
    "does not refresh after failed %s",
    async (action) => {
      api.fetchFillups.mockResolvedValue(
        page(action === "create" ? [] : [1], null),
      );
      const { container } = render(Dashboard);
      await waitFor(() => expect(api.fetchStats).toHaveBeenCalledTimes(1));
      await waitFor(() => expect(fillups.getLoading()).toBe(false));
      if (action === "create") {
        api.createFillup.mockRejectedValueOnce(new Error("offline"));
        await fireEvent.click(
          screen.getByRole("button", { name: "Add fill-up" }),
        );
        for (const [id, value] of [
          ["q-odometer", "11000"],
          ["q-fuel", "40"],
          ["q-total", "80"],
        ])
          await fireEvent.input(container.querySelector(`#${id}`)!, {
            target: { value },
          });
        await fireEvent.submit(container.querySelector("#quick-fill-form")!);
        await waitFor(() =>
          expect(container.querySelector(".save-error")).toBeTruthy(),
        );
      } else {
        await fireEvent.click(container.querySelector(".fillup-card")!);
        if (action === "update") {
          api.updateFillup.mockRejectedValueOnce(new Error("offline"));
          await fireEvent.submit(container.querySelector("#fillup-form")!);
          await waitFor(() =>
            expect(container.querySelector(".save-error")).toBeTruthy(),
          );
        } else {
          api.deleteFillup.mockRejectedValueOnce(new Error("offline"));
          await fireEvent.click(
            container.querySelector("#fillup-form .btn-danger")!,
          );
          await fireEvent.click(
            container.querySelector(".modal-dialog[open] .btn-danger")!,
          );
          await waitFor(() => expect(api.deleteFillup).toHaveBeenCalledOnce());
        }
      }
      expect(api.fetchStats).toHaveBeenCalledTimes(1);
      expect(api.fetchFillups).toHaveBeenCalledTimes(1);
    },
  );
});

describe("dashboard independent endless list", () => {
  it.each([true, false])(
    "uses the correct observer root (desktop=%s)",
    async (isDesktop) => {
      desktop = isDesktop;
      const { container } = await mount("older");
      expect(observers.at(-1)!.options).toMatchObject({
        root: isDesktop ? container.querySelector(".fillups-column") : null,
        rootMargin: "0px 0px 240px 0px",
      });
    },
  );
  it("guards duplicate intersections and keeps loaded cards during continuation", async () => {
    const pending = deferred<FillupPage>();
    const { container } = await mount("older");
    api.fetchFillups.mockReturnValueOnce(pending.promise);
    intersect(undefined, false);
    expect(api.fetchFillups).toHaveBeenCalledTimes(1);
    intersect();
    intersect();
    await waitFor(() => expect(api.fetchFillups).toHaveBeenCalledTimes(2));
    expect(container.querySelectorAll(".fillup-card")).toHaveLength(2);
    expect(screen.getByRole("status").textContent).toContain(
      "Loading older fill-ups",
    );
    pending.resolve(page([0], null));
    await waitFor(() =>
      expect(screen.queryByTestId("fillups-sentinel")).toBeNull(),
    );
    expect(api.fetchStats).toHaveBeenCalledTimes(1);
  });
  it("retries continuation failures without refreshing statistics", async () => {
    const { container } = await mount("retry");
    api.fetchFillups
      .mockRejectedValueOnce(new Error("offline"))
      .mockResolvedValueOnce(page([0], null));
    intersect();
    await fireEvent.click(
      await screen.findByRole("button", { name: "Try again" }),
    );
    await waitFor(() =>
      expect(container.querySelectorAll(".fillup-card")).toHaveLength(3),
    );
    expect(api.fetchFillups.mock.calls).toEqual([
      [1],
      [1, "retry"],
      [1, "retry"],
    ]);
    expect(api.fetchStats).toHaveBeenCalledTimes(1);
  });
  it("disconnects on root changes and unmount, ignoring stale callbacks", async () => {
    const rendered = await mount("older");
    const first = observers.at(-1)!;
    desktop = true;
    mediaListeners.forEach((cb) => cb());
    await waitFor(() => expect(first.disconnect).toHaveBeenCalledOnce());
    await waitFor(() =>
      expect(observers.at(-1)!.options?.root).toBe(
        rendered.container.querySelector(".fillups-column"),
      ),
    );
    const last = observers.at(-1)!;
    rendered.unmount();
    expect(last.disconnect).toHaveBeenCalledOnce();
    intersect(first);
    expect(api.fetchFillups).toHaveBeenCalledTimes(1);
  });
  it("formats absolute odometers using locale with no unit", async () => {
    preferences.locale = "de-DE";
    const { container } = await mount();
    expect(container.querySelector(".fillup-abs")?.textContent).toBe("10.000");
  });
});
