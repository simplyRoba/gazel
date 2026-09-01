import { cleanup, render } from "@testing-library/svelte";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import ChartCardFixture from "./ChartCard.test.svelte";

beforeEach(() => {
  vi.stubGlobal(
    "ResizeObserver",
    class {
      observe() {}
      unobserve() {}
      disconnect() {}
    },
  );
});

afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
});

describe("ChartCard", () => {
  const singlePoint = [{ x: 1, y: 10 }];

  it("requires two data points by default", () => {
    const { container } = render(ChartCardFixture, {
      props: { data: singlePoint },
    });

    expect(container.querySelector(".chart-empty")).not.toBeNull();
    expect(container.querySelector(".chart-container")).toBeNull();
  });

  it("renders one data point when the minimum is one", () => {
    const { container } = render(ChartCardFixture, {
      props: { data: singlePoint, minimumDataPoints: 1 },
    });

    expect(container.querySelector(".chart-container")).not.toBeNull();
    expect(container.querySelector(".chart-empty")).toBeNull();
  });
});
