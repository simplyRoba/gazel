import { vi } from "vitest";

vi.mock("#lib/chart-motion.svelte.js", () => ({
  tweenedValues: (target: () => number[]) => ({
    get current() {
      return target();
    },
  }),
}));
