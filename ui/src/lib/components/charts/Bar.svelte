<script lang="ts">
  import { getLayerCakeContext } from "layercake";
  import type { ChartScales } from "#lib/charts.js";

  const chart = getLayerCakeContext<ChartScales, Record<string, unknown>[]>();

  let { fill = "var(--color-accent)" } = $props();

  const bandwidth = $derived(
    "bandwidth" in chart.xScale ? chart.xScale.bandwidth() : 20,
  );
</script>

<g class="bars">
  {#each chart.data as d, i (i)}
    {@const x = chart.xGet(d)}
    {@const y = chart.yGet(d)}
    {@const barHeight = chart.height - y}
    {#if barHeight > 0}
      <rect {x} {y} width={bandwidth} height={barHeight} {fill} />
    {/if}
  {/each}
</g>
