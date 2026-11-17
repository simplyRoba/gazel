import { cleanup, fireEvent, render, waitFor } from "@testing-library/svelte";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import CostChart from "./CostChart.svelte";
import DistanceChart from "./DistanceChart.svelte";
import ChartsPanel from "./ChartsPanel.svelte";
import {
  hoverChart,
  mockChartDimensions,
  periodSeries,
  windowStart,
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
  monthly: periodSeries([
    { period: "2026-01", value: 123.45 },
    { period: "2026-03", value: 987.65 },
  ]),
  yearly: periodSeries([
    { period: "2020", value: 234.56 },
    { period: "2026", value: 876.54 },
  ]),
};

describe("server period charts", () => {
  it.each(["cost", "distance"] as const)(
    "persists clamped calendar selections across later refreshes",
    async (component) => {
      const points = Array.from({ length: 156 }, (_, i) => ({
        period: `${2010 + Math.floor(i / 12)}-${String((i % 12) + 1).padStart(2, "0")}`,
        value: i + 1,
      }));
      const modes = (monthly: typeof points) => ({
        monthly: periodSeries(monthly),
        yearly: periodSeries([{ period: "2022", value: 100 }]),
      });
      const props = {
        series: modes(points),
        currency: "USD",
        distanceUnit: "km",
        vehicleId: 1,
      };
      const { container, getByRole, rerender } =
        component === "cost"
          ? render(CostChart, props)
          : render(DistanceChart, props);
      await fireEvent.click(getByRole("button", { name: "Previous year" }));
      expect(windowStart(container)).toBe("Jan 21");
      await rerender({ ...props, series: modes(points.slice(0, 144)) });
      expect(windowStart(container)).toBe("Jan 21");
      await rerender(props);
      expect(windowStart(container)).toBe("Jan 22");
      await fireEvent.click(getByRole("button", { name: "Previous year" }));
      await fireEvent.click(getByRole("button", { name: "Previous year" }));
      expect(windowStart(container)).toBe("Jan 20");
      await rerender({ ...props, series: modes(points.slice(132)) });
      expect(windowStart(container)).toBe("Jan 21");
      await rerender(props);
      expect(windowStart(container)).toBe("Jan 21");
    },
  );
  it("navigates twelve-month windows, preserves them across modes, and shows all years", async () => {
    const monthly = Array.from({ length: 156 }, (_, i) => ({
      period: `${2010 + Math.floor(i / 12)}-${String((i % 12) + 1).padStart(2, "0")}`,
      value: i + 1,
    }));
    const yearly = Array.from({ length: 13 }, (_, i) => ({
      period: String(2010 + i),
      value: i + 1,
    }));
    const { container, getByRole, queryByRole } = render(CostChart, {
      series: { monthly: periodSeries(monthly), yearly: periodSeries(yearly) },
      currency: "USD",
      vehicleId: 1,
    });
    const previous = getByRole("button", { name: "Previous year" });
    for (const name of ["Next month", "Next year"])
      expect(
        (getByRole("button", { name }) as HTMLButtonElement).disabled,
      ).toBe(true);
    expect(windowStart(container)).toBe("Jan 22");
    await fireEvent.click(previous);
    expect(windowStart(container)).toBe("Jan 21");
    await fireEvent.click(getByRole("button", { name: "Yearly" }));
    expect(queryByRole("button", { name: "Previous year" })).toBeNull();
    expect(queryByRole("button", { name: "Previous month" })).toBeNull();
    await waitFor(() =>
      expect(container.querySelectorAll(".bars rect")).toHaveLength(13),
    );
    await fireEvent.click(getByRole("button", { name: "Monthly" }));
    expect(windowStart(container)).toBe("Jan 21");
    for (let i = 0; i < 20; i++) {
      const button = getByRole("button", {
        name: "Previous year",
      }) as HTMLButtonElement;
      if (button.disabled) break;
      await fireEvent.click(button);
    }
    expect(windowStart(container)).toBe("Jan 10");
    expect(container.querySelectorAll(".bars rect")).toHaveLength(12);
    for (const name of ["Previous month", "Previous year"])
      expect(
        (getByRole("button", { name }) as HTMLButtonElement).disabled,
      ).toBe(true);
    await fireEvent.click(getByRole("button", { name: "Next year" }));
    expect(windowStart(container)).toBe("Jan 11");
  });
  it.each(["cost", "distance"] as const)(
    "shows all months without navigation for a %s history of at most 12 months",
    async (component) => {
      const monthly = Array.from({ length: 12 }, (_, i) => ({
        period: `2025-${String(i + 1).padStart(2, "0")}`,
        value: i + 1,
      }));
      const props = {
        series: {
          monthly: periodSeries(monthly),
          yearly: periodSeries([{ period: "2025", value: 78 }]),
        },
        currency: "USD",
        distanceUnit: "km",
        vehicleId: 1,
      };
      const { container, queryByRole } =
        component === "cost"
          ? render(CostChart, props)
          : render(DistanceChart, props);
      await waitFor(() =>
        expect(container.querySelectorAll(".axis-x > g")).toHaveLength(12),
      );
      expect(windowStart(container)).toBe("Jan 25");
      for (const name of [
        "Previous month",
        "Next month",
        "Previous year",
        "Next year",
      ])
        expect(queryByRole("button", { name })).toBeNull();
    },
  );
  it.each(["cost", "distance"] as const)(
    "steps the %s chart one month at a time without a visible date range",
    async (component) => {
      const monthly = Array.from({ length: 35 }, (_, i) => ({
        period: `${2020 + Math.floor(i / 12)}-${String((i % 12) + 1).padStart(2, "0")}`,
        value: i + 1,
      }));
      const props = {
        series: {
          monthly: periodSeries(monthly),
          yearly: periodSeries([{ period: "2022", value: 1 }]),
        },
        currency: "USD",
        distanceUnit: "km",
        vehicleId: 1,
      };
      const { container, getByRole } =
        component === "cost"
          ? render(CostChart, props)
          : render(DistanceChart, props);
      expect(windowStart(container)).toBe("Dec 21");
      expect(container.textContent).not.toMatch(/Dec 2021|Nov 2022/);
      await fireEvent.click(getByRole("button", { name: "Previous month" }));
      expect(windowStart(container)).toBe("Nov 21");
      await fireEvent.click(getByRole("button", { name: "Next month" }));
      await fireEvent.click(getByRole("button", { name: "Previous year" }));
      await fireEvent.click(getByRole("button", { name: "Next month" }));
      expect(windowStart(container)).toBe("Jan 21");
      expect(container.querySelectorAll(".axis-x > g")).toHaveLength(12);
    },
  );
  it("renders every yearly distance point across more than twelve years", async () => {
    const yearly = Array.from({ length: 13 }, (_, i) => ({
      period: String(2010 + i),
      value: (i + 1) * 10,
    }));
    const { container, getByRole, queryByRole } = render(DistanceChart, {
      series: { monthly: series.monthly, yearly: periodSeries(yearly) },
      distanceUnit: "km",
      vehicleId: 1,
    });
    await fireEvent.click(getByRole("button", { name: "Yearly" }));
    await waitFor(() =>
      expect(container.querySelectorAll(".axis-x > g")).toHaveLength(13),
    );
    expect(queryByRole("button", { name: "Previous month" })).toBeNull();
    expect(queryByRole("button", { name: "Next month" })).toBeNull();
    const width = Number(
      container
        .querySelector('rect[role="presentation"]')!
        .getAttribute("width"),
    );
    expect(width).toBeGreaterThan(0);
    for (const [index, point] of yearly.entries()) {
      await hoverChart(container, (index * width) / (yearly.length - 1));
      expect(
        container.querySelector('text[text-anchor="end"][font-size="10"]')!
          .textContent,
      ).toBe(point.period);
      expect(
        container.querySelector('text[text-anchor="end"][font-size="12"]')!
          .textContent,
      ).toBe(`${point.value}.0 km`);
    }
  });

  it("plots exact supplied cost values in both modes without reconstructing data or omission captions", async () => {
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
    expect(container.querySelector(".chart-caption")).toBeNull();
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
    expect(container.querySelector(".chart-caption")).toBeNull();
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
    expect(container.querySelector(".chart-caption")).toBeNull();
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
    expect(container.querySelector(".chart-caption")).toBeNull();
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
