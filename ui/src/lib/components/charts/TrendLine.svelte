<script lang="ts">
  import { getLayerCakeContext } from "layercake";
  import type { ChartScales, TimeSeriesPoint } from "#lib/charts.js";

  const chart = getLayerCakeContext<ChartScales, Record<string, unknown>[]>();

  let {
    points,
    stroke = "var(--color-text-tertiary)",
    strokeWidth = 1.5,
    opacity = 0.6,
  }: {
    points: TimeSeriesPoint[] | null;
    stroke?: string;
    strokeWidth?: number;
    opacity?: number;
  } = $props();

  // These are the full-population fit endpoints, not a fit of retained prices.
  const trendPath = $derived(
    points?.length === 2
      ? `M${chart.xGet(points[0])},${chart.yGet(points[0])} L${chart.xGet(points[1])},${chart.yGet(points[1])}`
      : "",
  );
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
