import { untrack } from "svelte";
import { cubicOut } from "svelte/easing";
import { Tween } from "svelte/motion";

function resample(values: number[], length: number): number[] {
  if (values.length === 1) return Array.from({ length }, () => values[0]);
  return Array.from({ length }, (_, i) => {
    const pos = length === 1 ? 0 : (i / (length - 1)) * (values.length - 1);
    const lo = Math.floor(pos);
    const hi = Math.min(lo + 1, values.length - 1);
    return values[lo] + (values[hi] - values[lo]) * (pos - lo);
  });
}

export function tweenedValues(target: () => number[], baseline: () => number) {
  const tween = new Tween<number[]>([], { duration: 400, easing: cubicOut });
  let previousBase = 0;
  $effect(() => {
    const next = target();
    const base = baseline();
    if (base <= 0) return;
    untrack(() => {
      let previous = tween.current;
      if (previousBase > 0 && base !== previousBase) {
        previous = previous.map((v) => (v * base) / previousBase);
        void tween.set(previous, { duration: 0 });
      }
      previousBase = base;
      if (previous.length === 0) {
        void tween.set(
          next.map((v) => base + (v - base) * 0.5),
          { duration: 0 },
        );
      } else if (previous.length !== next.length) {
        void tween.set(resample(previous, next.length), { duration: 0 });
      }
    });
    const frame = requestAnimationFrame(() => void tween.set(next));
    return () => cancelAnimationFrame(frame);
  });
  return tween;
}
