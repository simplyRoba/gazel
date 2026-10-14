import { cleanup, fireEvent, render, waitFor } from "@testing-library/svelte";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import CostChart from "./CostChart.svelte";
import DistanceChart from "./DistanceChart.svelte";
import ChartsPanel from "./ChartsPanel.svelte";
import {
  hoverChart,
  mockChartDimensions,
  periodSeries,
} from "./chart-test-helpers.js";

const globalSettings = vi.hoisted(() => ({
  locale: "en",
  currency: "EUR",
  distance_unit: "mi",
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

const series = {
  monthly: periodSeries(
    [
      { period: "2026-01", value: 123.45 },
      { period: "2026-03", value: 987.65 },
    ],
    true,
  ),
  yearly: periodSeries(
    [
      { period: "2020", value: 234.56 },
      { period: "2026", value: 876.54 },
    ],
    true,
  ),
};

describe("server period charts", () => {
  it("plots exact supplied cost values in both modes, without filling gaps or slicing, and labels omitted history", async () => {
    const { container, getByRole, rerender } = render(CostChart, {
      series,
      currency: "USD",
    });
    await waitFor(() =>
      expect(container.querySelectorAll(".bars rect")).toHaveLength(2),
    );
    expect(container.querySelector(".axis-x")!.textContent).not.toContain(
      "Feb",
    );
    expect(container.querySelector(".chart-caption")!.textContent).toBe(
      "Showing 2026-01–2026-03; older periods omitted",
    );
    await hoverChart(container, 0);
    expect(container.textContent).toContain("$123.45");
    await hoverChart(container, 600);
    expect(container.textContent).toContain("$987.65");
    await fireEvent.click(getByRole("button", { name: "Yearly" }));
    await waitFor(() =>
      expect(container.querySelector(".axis-x")!.textContent).toContain("2020"),
    );
    expect(container.querySelectorAll(".bars rect")).toHaveLength(2);
    expect(container.querySelector(".axis-x")!.textContent).not.toContain(
      "2021",
    );
    expect(container.querySelector(".chart-caption")!.textContent).toBe(
      "Showing 2020–2026; older periods omitted",
    );
    await hoverChart(container, 0);
    expect(container.textContent).toContain("$234.56");
    await hoverChart(container, 600);
    expect(container.textContent).toContain("$876.54");
    const bars = [...container.querySelectorAll(".bars rect")].map(
      (r) => r.outerHTML,
    );
    globalSettings.currency = "GBP";
    await rerender({ series, currency: "USD" });
    expect(
      [...container.querySelectorAll(".bars rect")].map((r) => r.outerHTML),
    ).toEqual(bars);
    expect(container.querySelector(".chart-title")!.textContent).toBe(
      "Yearly cost (USD)",
    );
  });

  it("plots exact supplied distance values in both modes with a zero-based response-unit axis", async () => {
    const { container, getByRole, rerender } = render(DistanceChart, {
      series,
      distanceUnit: "km",
    });
    await waitFor(() =>
      expect(container.querySelectorAll("path")).toHaveLength(2),
    );
    expect(container.querySelector(".axis-x")!.textContent).not.toContain(
      "Feb",
    );
    expect(container.querySelector(".axis-y")!.textContent).toContain("0 km");
    expect(container.querySelector(".chart-title")!.textContent).toBe(
      "Monthly distance (km)",
    );
    expect(container.querySelector(".chart-caption")!.textContent).toBe(
      "Showing 2026-01–2026-03; older periods omitted",
    );
    await hoverChart(container, 0);
    expect(container.textContent).toContain("123.5 km");
    await hoverChart(container, 600);
    expect(container.textContent).toContain("987.7 km");
    // Fractional inputs are not rounded by the mapper: y = 140 - value/987.65*140.
    const line = container.querySelector('path[fill="none"]')!;
    expect(line.getAttribute("d")).toContain(
      (140 - (123.45 / 987.65) * 140).toFixed(3),
    );
    await fireEvent.click(getByRole("button", { name: "Yearly" }));
    await waitFor(() =>
      expect(container.querySelector(".axis-x")!.textContent).toContain("2020"),
    );
    expect(container.querySelector(".axis-x")!.textContent).not.toContain(
      "2021",
    );
    expect(container.querySelector(".chart-caption")!.textContent).toBe(
      "Showing 2020–2026; older periods omitted",
    );
    await hoverChart(container, 0);
    expect(container.textContent).toContain("234.6 km");
    await hoverChart(container, 600);
    expect(container.textContent).toContain("876.5 km");
    expect(line.getAttribute("d")).toContain(
      (140 - (234.56 / 876.54) * 140).toFixed(3),
    );
    const paths = [...container.querySelectorAll("path")].map((p) =>
      p.getAttribute("d"),
    );
    globalSettings.distance_unit = "mi";
    await rerender({ series, distanceUnit: "km" });
    expect(
      [...container.querySelectorAll("path")].map((p) => p.getAttribute("d")),
    ).toEqual(paths);
    expect(container.querySelector(".chart-title")!.textContent).toBe(
      "Yearly distance (km)",
    );
    expect(container.querySelector(".axis-y")!.textContent).not.toContain("mi");
  });

  it("retains cost >=1 and distance >=2 thresholds, null cards, and hides labels for complete history", async () => {
    const onePeriod = {
      monthly: periodSeries([{ period: "2026-01", value: 10 }]),
      yearly: periodSeries([{ period: "2026", value: 10 }]),
    };
    const cost = render(CostChart, { series: onePeriod, currency: "USD" });
    await waitFor(() =>
      expect(cost.container.querySelectorAll(".bars rect")).toHaveLength(1),
    );
    expect(cost.container.querySelector(".chart-caption")).toBeNull();
    await cost.rerender({ series: null });
    expect(cost.container.textContent).toContain(
      "More fill-ups needed for chart",
    );
    const distance = render(DistanceChart, {
      series: onePeriod,
      distanceUnit: "km",
    });
    expect(distance.container.querySelector("svg")).toBeNull();
    expect(distance.container.textContent).toContain(
      "More fill-ups needed for chart",
    );
    await distance.rerender({ series: null });
    expect(distance.container.querySelector("svg")).toBeNull();
    const panel = render(ChartsPanel, {
      charts: null,
      distanceUnit: "km",
      volumeUnit: "l",
      currency: "USD",
    });
    expect(panel.container.querySelectorAll(".chart-empty")).toHaveLength(3);
  });
});
