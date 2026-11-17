<script lang="ts">
  import { getLayerCakeContext } from "layercake";
  import type { ChartScales } from "#lib/charts.js";
  import { tweenedValues } from "#lib/chart-motion.svelte.js";

  const chart = getLayerCakeContext<ChartScales, Record<string, unknown>[]>();

  let { fill = "var(--color-accent)" } = $props();

  const bandwidth = $derived(
    "bandwidth" in chart.xScale ? chart.xScale.bandwidth() : 20,
  );
  const ys = tweenedValues(
    () => chart.data.map((d) => chart.yGet(d) as number),
    () => chart.height,
  );
</script>

<g class="bars">
  {#each chart.data as d, i (i)}
    {#if chart.height - chart.yGet(d) > 0}
      {@const y = ys.current[i] ?? chart.height}
      <rect
        x={chart.xGet(d)}
        {y}
        width={bandwidth}
        height={Math.max(0, chart.height - y)}
        {fill}
      />
    {/if}
  {/each}
</g>
