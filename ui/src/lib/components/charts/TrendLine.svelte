<script lang="ts">
  import { getContext } from "svelte";
  import type { LayerCakeContext } from "$lib/charts";

  const { data, xGet, yGet } = getContext("LayerCake") as LayerCakeContext;

  let {
    stroke = "var(--color-text-tertiary)",
    strokeWidth = 1.5,
    opacity = 0.6,
  } = $props();

  /* eslint-disable @typescript-eslint/no-explicit-any */
  const trendPath = $derived.by(() => {
    const pts = $data;
    if (pts.length < 2) return "";

    const xs = pts.map((d: Record<string, any>) => $xGet(d));
    const ys = pts.map((d: Record<string, any>) => $yGet(d));
    const n = xs.length;

    const sumX = xs.reduce((a: number, b: number) => a + b, 0);
    const sumY = ys.reduce((a: number, b: number) => a + b, 0);
    const sumXY = xs.reduce(
      (a: number, x: number, i: number) => a + x * ys[i],
      0,
    );
    const sumX2 = xs.reduce((a: number, x: number) => a + x * x, 0);

    const denom = n * sumX2 - sumX * sumX;
    if (denom === 0) return "";

    const slope = (n * sumXY - sumX * sumY) / denom;
    const intercept = (sumY - slope * sumX) / n;

    const x1 = xs[0];
    const x2 = xs[n - 1];
    const y1 = intercept + slope * x1;
    const y2 = intercept + slope * x2;

    return `M${x1},${y1} L${x2},${y2}`;
  });
  /* eslint-enable @typescript-eslint/no-explicit-any */
</script>

{#if trendPath}
  <path
    d={trendPath}
    fill="none"
    {stroke}
    stroke-width={strokeWidth}
    stroke-dasharray="6,4"
    {opacity}
  />
{/if}
