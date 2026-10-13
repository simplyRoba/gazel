<script lang="ts">
  import { getLayerCakeContext } from "layercake";
  import type { ChartScales } from "#lib/charts.js";

  const chart = getLayerCakeContext<ChartScales>();

  let { ticks = 4, format = (v: number) => String(v) } = $props();
</script>

<g class="axis-y">
  {#each chart.yScale.ticks(ticks) as tick (tick)}
    <g transform="translate(0, {chart.yScale(tick)})">
      <line
        x1={0}
        x2={chart.width}
        stroke="var(--color-border-subtle)"
        stroke-dasharray="2,4"
        opacity="0.5"
      />
      <text
        x={-8}
        dy="0.35em"
        text-anchor="end"
        fill="var(--color-text-tertiary)"
        font-size="11"
        font-family="var(--font-family)"
      >
        {format(tick)}
      </text>
    </g>
  {/each}
</g>
