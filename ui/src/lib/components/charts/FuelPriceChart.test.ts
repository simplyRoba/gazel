import { cleanup, fireEvent, render, waitFor } from "@testing-library/svelte";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import type { Fillup } from "#lib/api.js";
import FuelPriceChart from "./FuelPriceChart.svelte";

function fillup(overrides: Partial<Fillup> = {}): Fillup {
  return {
    id: 1,
    vehicle_id: 1,
    date: "2026-03-01",
    odometer: 0,
    fuel_amount: 20,
    fuel_unit: "l",
    cost: 40,
    currency: "USD",
    is_full_tank: false,
    is_missed: false,
    station: null,
    notes: null,
    created_at: "",
    updated_at: "",
    ...overrides,
  };
}

beforeEach(() => {
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
});
afterEach(() => {
  cleanup();
  vi.restoreAllMocks();
  vi.unstubAllGlobals();
});

describe("FuelPriceChart", () => {
  it("updates from a single dot to a chronological line and trend, then another vehicle and empty data", async () => {
    const first = fillup();
    const { container, rerender } = render(FuelPriceChart, {
      fillups: [first],
      currency: "USD",
      volumeUnit: "l",
    });
    await waitFor(() =>
      expect(container.querySelector(".line-point")).not.toBeNull(),
    );
    expect(container.querySelector("path")).toBeNull();
    await rerender({
      fillups: [
        fillup({ id: 2, date: "2026-04-01", cost: 60, is_missed: true }),
        first,
      ],
    });
    await waitFor(() =>
      expect(container.querySelectorAll("path")).toHaveLength(2),
    );
    expect(container.querySelector(".line-point")).toBeNull();
    for (const path of container.querySelectorAll("path")) {
      expect(path.getAttribute("d")).toMatch(/^M/);
      expect(path.getAttribute("d")).not.toMatch(/NaN|Infinity/);
    }
    const overlay = container.querySelector<SVGRectElement>(
      'rect[role="presentation"]',
    )!;
    overlay.getBoundingClientRect = () => ({ left: 0 }) as DOMRect;
    await fireEvent.mouseMove(overlay, { clientX: 0 });
    expect(container.textContent).toContain("$2.00/L");
    await fireEvent.mouseMove(overlay, { clientX: 600 });
    expect(container.textContent).toContain("$3.00/L");

    await rerender({
      fillups: [
        fillup({ vehicle_id: 2, fuel_unit: "gal", fuel_amount: 1, cost: 8 }),
      ],
      volumeUnit: "gal",
    });
    await fireEvent.mouseMove(overlay, { clientX: 200 });
    expect(container.textContent).toContain("$8.00/gal");
    await rerender({ volumeUnit: "l" });
    expect(container.textContent).toContain("$2.11/L");
    await rerender({ fillups: [] });
    expect(container.querySelector(".chart-empty")).not.toBeNull();
    expect(container.querySelector("svg")).toBeNull();
  });

  it("renders a finite dot and tooltip for a free fill-up", async () => {
    const { container } = render(FuelPriceChart, {
      fillups: [fillup({ cost: 0 })],
      currency: "USD",
      volumeUnit: "l",
    });
    await waitFor(() =>
      expect(container.querySelector(".line-point")).not.toBeNull(),
    );
    const dot = container.querySelector(".line-point")!;
    expect(Number.isFinite(Number(dot.getAttribute("cy")))).toBe(true);
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
