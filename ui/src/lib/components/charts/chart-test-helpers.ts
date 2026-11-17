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

export function windowStart(container: Element): string | undefined {
  return container.querySelector(".axis-x text")?.textContent?.trim();
}

export function periodSeries(points: PeriodSeries["points"]): PeriodSeries {
  return { points };
}

export function priceSeries(
  points: FuelPricePoint[],
  trend: FuelPriceSeries["trend"] = null,
): FuelPriceSeries {
  return { points, trend };
}
