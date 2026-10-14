import { fireEvent } from "@testing-library/svelte";
import { vi } from "vitest";
import type {
  FuelPricePoint,
  FuelPriceSeries,
  PeriodSeries,
} from "#lib/api.js";

export function mockChartDimensions(): void {
  vi.spyOn(HTMLElement.prototype, "clientWidth", "get").mockReturnValue(640);
  vi.spyOn(HTMLElement.prototype, "clientHeight", "get").mockReturnValue(180);
  vi.stubGlobal(
    "ResizeObserver",
    class {
      observe = vi.fn();
      unobserve = vi.fn();
      disconnect = vi.fn();
    },
  );
}

export async function hoverChart(
  container: HTMLElement,
  clientX: number,
): Promise<void> {
  const overlay = container.querySelector<SVGRectElement>(
    'rect[role="presentation"]',
  )!;
  overlay.getBoundingClientRect = () => ({ left: 0 }) as DOMRect;
  await fireEvent.mouseMove(overlay, { clientX });
}

export function periodSeries(
  points: PeriodSeries["points"],
  omitted = false,
): PeriodSeries {
  const range = points.length
    ? { start: points[0].period, end: points[points.length - 1].period }
    : null;
  return {
    points,
    source_count: points.length,
    source_range: range,
    displayed_range: range,
    omitted_older_periods: omitted,
  };
}

export function priceSeries(
  points: FuelPricePoint[],
  trend: FuelPriceSeries["trend"] = null,
): FuelPriceSeries {
  const range = points.length
    ? { start: points[0].date, end: points[points.length - 1].date }
    : null;
  return {
    points,
    source_count: points.length,
    reduced: false,
    source_range: range,
    displayed_range: range,
    trend,
  };
}
