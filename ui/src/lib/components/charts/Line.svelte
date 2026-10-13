<script lang="ts">
  import { getLayerCakeContext } from "layercake";
  import { line, curveMonotoneX } from "d3-shape";
  import type { ChartScales } from "#lib/charts.js";

  const chart = getLayerCakeContext<ChartScales, Record<string, unknown>[]>();

  let { stroke = "var(--color-accent)", strokeWidth = 2 } = $props();

  const pathD = $derived(
    line<Record<string, unknown>>()
      .x((d) => chart.xGet(d))
      .y((d) => chart.yGet(d))
      .curve(curveMonotoneX)(chart.data) ?? "",
  );
</script>

{#if chart.data.length === 1}
  <circle
    class="line-point"
    cx={chart.xGet(chart.data[0])}
    cy={chart.yGet(chart.data[0])}
    r={3}
    fill={stroke}
  />
{:else}
  <path
    d={pathD}
    fill="none"
    {stroke}
    stroke-width={strokeWidth}
    stroke-linejoin="round"
    stroke-linecap="round"
  />
{/if}
