import { flushSync } from "svelte";
import { describe, expect, it, vi } from "vitest";

vi.unmock("#lib/chart-motion.svelte.js");
vi.stubGlobal("matchMedia", () => ({
  matches: false,
  addEventListener: () => {},
  removeEventListener: () => {},
}));
const { tweenedValues } = await import("./chart-motion.svelte.js");

describe("tweened chart values", () => {
  it("enters halfway from the baseline, eases to the target, and morphs between lengths", async () => {
    let values = $state([10, 20]);
    let tween!: { current: number[] };
    const destroy = $effect.root(() => {
      tween = tweenedValues(
        () => values,
        () => 100,
      );
    });
    flushSync();
    expect(tween.current).toEqual([55, 60]);
    await vi.waitFor(() => expect(tween.current).toEqual([10, 20]));

    values = [50, 60];
    flushSync();
    await vi.waitFor(() => {
      expect(tween.current[0]).toBeGreaterThan(10);
      expect(tween.current[0]).toBeLessThan(50);
    });
    await vi.waitFor(() => expect(tween.current).toEqual([50, 60]));

    values = [1, 2, 3];
    flushSync();
    expect(tween.current).toEqual([50, 55, 60]);
    await vi.waitFor(() => expect(tween.current).toEqual([1, 2, 3]));

    values = [7];
    flushSync();
    expect(tween.current).toEqual([1]);
    destroy();
  });

  it("waits for a measured baseline before animating", () => {
    let baseline = $state(0);
    let tween!: { current: number[] };
    const destroy = $effect.root(() => {
      tween = tweenedValues(
        () => [10],
        () => baseline,
      );
    });
    flushSync();
    expect(tween.current).toEqual([]);
    baseline = 140;
    flushSync();
    expect(tween.current).toEqual([75]);
    destroy();
  });

  it("rescales in-flight values when the chart height changes", () => {
    let baseline = $state(100);
    let tween!: { current: number[] };
    const destroy = $effect.root(() => {
      tween = tweenedValues(
        () => [baseline * 0.2],
        () => baseline,
      );
    });
    flushSync();
    expect(tween.current).toEqual([60]);
    baseline = 200;
    flushSync();
    expect(tween.current).toEqual([120]);
    destroy();
  });
});
