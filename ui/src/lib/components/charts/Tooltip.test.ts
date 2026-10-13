import { cleanup, fireEvent, render } from "@testing-library/svelte";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import TooltipFixture from "./Tooltip.test.svelte";

beforeEach(() => {
  vi.spyOn(HTMLElement.prototype, "clientWidth", "get").mockReturnValue(100);
  vi.spyOn(HTMLElement.prototype, "clientHeight", "get").mockReturnValue(50);
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

describe("Tooltip", () => {
  it("selects the nearest point when touch starts", async () => {
    const { container } = render(TooltipFixture);
    const overlay = container.querySelector<SVGRectElement>(
      'rect[role="presentation"]',
    );
    expect(overlay).not.toBeNull();
    overlay!.getBoundingClientRect = () => ({ left: 0 }) as DOMRect;

    await fireEvent.touchStart(overlay!, {
      touches: [{ clientX: 75, clientY: 10 }],
    });

    expect(container.textContent).toContain("Second");
    expect(container.textContent).toContain("2");
  });

  it("updates hovered details and coordinates when data and chart size change", async () => {
    const { container, rerender } = render(TooltipFixture);
    const overlay = container.querySelector<SVGRectElement>(
      'rect[role="presentation"]',
    )!;
    overlay.getBoundingClientRect = () => ({ left: 0 }) as DOMRect;
    await fireEvent.mouseMove(overlay, { clientX: 80 });
    expect(container.querySelector("circle")?.getAttribute("cx")).toBe("80");

    await rerender({
      points: [
        { x: 10, y: 20, date: "Updated first", value: 3 },
        { x: 60, y: 40, date: "Updated second", value: 4 },
      ],
      width: 200,
      height: 100,
    });

    expect(overlay.getAttribute("width")).toBe("200");
    expect(overlay.getAttribute("height")).toBe("100");
    expect(container.textContent).toContain("Updated second");
    expect(container.querySelector("circle")?.getAttribute("cx")).toBe("120");
    expect(container.querySelector("circle")?.getAttribute("cy")).toBe("80");
    expect(container.querySelector("text")?.getAttribute("x")).toBe("200");

    await fireEvent.mouseLeave(overlay);
    expect(container.querySelector("circle")).toBeNull();
    expect(container.querySelector("text")).toBeNull();
  });

  it("hides hovered details when the selected point no longer exists", async () => {
    const { container, rerender } = render(TooltipFixture);
    const overlay = container.querySelector<SVGRectElement>(
      'rect[role="presentation"]',
    )!;
    overlay.getBoundingClientRect = () => ({ left: 0 }) as DOMRect;
    await fireEvent.mouseMove(overlay, { clientX: 80 });

    await rerender({ points: [] });
    expect(container.querySelector("circle")).toBeNull();
    await fireEvent.mouseMove(overlay, { clientX: 20 });
    expect(container.querySelector("text")).toBeNull();
  });
});
