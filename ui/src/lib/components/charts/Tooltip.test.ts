import { fireEvent, render } from "@testing-library/svelte";
import { describe, expect, it } from "vitest";

import TooltipFixture from "./Tooltip.test.svelte";

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
});
