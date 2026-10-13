<script lang="ts">
  import { getLayerCakeContext } from "layercake";
  import { area, curveMonotoneX } from "d3-shape";
  import type { ChartScales } from "#lib/charts.js";

  const chart = getLayerCakeContext<ChartScales, Record<string, unknown>[]>();

  let { fill = "var(--color-accent-subtle)", opacity = 0.3 } = $props();

  const pathD = $derived(
    area<Record<string, unknown>>()
      .x((d) => chart.xGet(d))
      .y0(chart.height)
      .y1((d) => chart.yGet(d))
      .curve(curveMonotoneX)(chart.data) ?? "",
  );
</script>

{#if chart.data.length >= 2}
  <path d={pathD} {fill} {opacity} stroke="none" />
{/if}
