import { cleanup, fireEvent, render, waitFor } from "@testing-library/svelte";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import FuelPriceChart from "./FuelPriceChart.svelte";
import {
  hoverChart,
  mockChartDimensions,
  priceSeries,
} from "./chart-test-helpers.js";

const globalSettings = vi.hoisted(() => ({
  locale: "en",
  currency: "EUR",
  volume_unit: "gal",
}));
vi.mock("#lib/stores/settings.svelte.js", () => ({
  getSettings: () => globalSettings,
}));

beforeEach(mockChartDimensions);
afterEach(() => {
  cleanup();
  vi.restoreAllMocks();
  vi.unstubAllGlobals();
});

describe("FuelPriceChart", () => {
  it("shows the first recorded price as a finite dot, with response units in title and tooltip only", async () => {
    const { container } = render(FuelPriceChart, {
      series: priceSeries([{ date: "2026-03-01", value: 2 }]),
      currency: "USD",
      volumeUnit: "l",
    });
    await waitFor(() =>
      expect(container.querySelector(".line-point")).not.toBeNull(),
    );
    expect(container.querySelector("path")).toBeNull();
    expect(
      Number.isFinite(
        Number(container.querySelector(".line-point")!.getAttribute("cy")),
      ),
    ).toBe(true);
    expect(container.querySelector(".chart-title")!.textContent).toBe(
      "Fuel price (USD/L)",
    );
    expect(container.querySelector(".axis-y")!.textContent).toContain("$");
    expect(container.querySelector(".axis-y")!.textContent).not.toContain("/L");
    expect(container.querySelector(".axis-x")!.textContent).toContain("26");
    await hoverChart(container, 0);
    expect(container.textContent).toContain("$2.00/L");
    expect(container.textContent).toContain("Mar 1, 2026");
  });

  it("draws provided full-population endpoints rather than refitting a biased retained subset", async () => {
    const series = {
      ...priceSeries(
        [
          { date: "2026-03-01", value: 0 },
          { date: "2026-03-15", value: 100 },
          { date: "2026-04-01", value: 0 },
        ],
        [
          { date: "2026-03-01", value: 25 },
          { date: "2026-04-01", value: 25 },
        ],
      ),
      source_count: 500,
      reduced: true,
    };
    const { container, rerender } = render(FuelPriceChart, {
      series,
      currency: "USD",
      volumeUnit: "l",
    });
    const trendSelector = 'path[stroke-dasharray="6,4"]';
    await waitFor(() =>
      expect(container.querySelector(trendSelector)).not.toBeNull(),
    );
    // Inner chart height = 180 - top 10 - bottom 30 = 140; y-domain = [0,100].
    expect(container.querySelector(trendSelector)!.getAttribute("d")).toBe(
      "M0,105 L578,105",
    );
    expect(container.querySelector(".chart-caption")!.textContent).toBe(
      "Showing 3 of 500 recorded prices (2026-03-01–2026-04-01)",
    );
    const paths = [...container.querySelectorAll("path")].map((p) =>
      p.getAttribute("d"),
    );
    // An unrelated render or changed global settings cannot reinterpret this response.
    globalSettings.currency = "GBP";
    globalSettings.volume_unit = "gal";
    await rerender({ series, currency: "USD", volumeUnit: "l" });
    expect(
      [...container.querySelectorAll("path")].map((p) => p.getAttribute("d")),
    ).toEqual(paths);
    expect(container.querySelector(".chart-title")!.textContent).toBe(
      "Fuel price (USD/L)",
    );
    await hoverChart(container, 280);
    expect(container.textContent).toContain("$100.00/L");
    await rerender({ series: { ...series, trend: null } });
    expect(container.querySelector(trendSelector)).toBeNull();
    expect(container.querySelectorAll("path")).toHaveLength(1);
    await rerender({ series: null });
    expect(container.querySelector("svg")).toBeNull();
    expect(container.textContent).toContain("More fill-ups needed for chart");
  });

  it("updates supplied endpoints reactively and never synthesizes a fit for a null trend", async () => {
    const points = [
      { date: "2026-03-01", value: 1 },
      { date: "2026-04-01", value: 3 },
    ];
    const { container, rerender } = render(FuelPriceChart, {
      series: priceSeries(points),
      currency: "USD",
      volumeUnit: "gal",
    });
    await waitFor(() =>
      expect(container.querySelectorAll("path")).toHaveLength(1),
    );
    await rerender({ series: priceSeries(points, [points[0], points[1]]) });
    await waitFor(() =>
      expect(container.querySelectorAll("path")).toHaveLength(2),
    );
    const previous = container
      .querySelector('path[stroke-dasharray="6,4"]')!
      .getAttribute("d");
    await rerender({
      series: priceSeries(points, [
        { date: "2026-03-01", value: 2 },
        { date: "2026-04-01", value: 2 },
      ]),
    });
    expect(
      container
        .querySelector('path[stroke-dasharray="6,4"]')!
        .getAttribute("d"),
    ).not.toBe(previous);
    await hoverChart(container, 600);
    expect(container.textContent).toContain("$3.00/gal");
  });

  it("renders a finite dot and touch tooltip for an already eligible free recorded price", async () => {
    const { container } = render(FuelPriceChart, {
      series: priceSeries([{ date: "2026-03-01", value: 0 }]),
      currency: "USD",
      volumeUnit: "l",
    });
    await waitFor(() =>
      expect(container.querySelector(".line-point")).not.toBeNull(),
    );
    expect(
      Number.isFinite(
        Number(container.querySelector(".line-point")!.getAttribute("cy")),
      ),
    ).toBe(true);
    const overlay = container.querySelector<SVGRectElement>(
      'rect[role="presentation"]',
    )!;
    overlay.getBoundingClientRect = () => ({ left: 0 }) as DOMRect;
    await fireEvent.touchStart(overlay, {
      touches: [{ clientX: 200, clientY: 10 }],
    });
    expect(container.textContent).toContain("$0.00/L");
  });
});
