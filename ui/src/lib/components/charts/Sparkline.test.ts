import { cleanup, render } from "@testing-library/svelte";
import { afterEach, describe, expect, it } from "vitest";
import { toSparklineData } from "#lib/charts.js";
import Sparkline from "./Sparkline.svelte";

afterEach(cleanup);

describe("bounded sparkline geometry", () => {
  it("keeps original unequal sampled positions rather than spacing retained points equally", async () => {
    const data = toSparklineData({
      points: [
        { position: 0, value: 8 },
        { position: 0.1, value: 10 },
        { position: 0.9, value: 6 },
        { position: 1, value: 9 },
      ],
      source_count: 1000,
      reduced: true,
    });
    const { container, rerender } = render(Sparkline, { data });
    const path = container
      .querySelector('path[fill="none"]')!
      .getAttribute("d")!;
    // Bezier endpoints retain x=10 and x=90, not 33.3 and 66.7.
    expect(path).toContain(" 10,10");
    expect(path).toContain(" 90,90");
    expect(path).toContain(" 100,30");
    expect(path).not.toMatch(/NaN|Infinity/);
    await rerender({ data });
    expect(
      container.querySelector('path[fill="none"]')!.getAttribute("d"),
    ).toBe(path);
  });

  it("does not render trend paths below the existing two-point threshold", () => {
    const { container } = render(Sparkline, { data: [{ x: 0.5, y: 8 }] });
    expect(container.querySelector("path")).toBeNull();
  });
});
