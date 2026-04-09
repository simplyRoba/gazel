<script lang="ts">
  import { getContext } from "svelte";
  import type { LayerCakeContext } from "$lib/charts";
  import type { ScaleLinear } from "d3-scale";

  const { yScale, width } = getContext("LayerCake") as LayerCakeContext;

  let { ticks = 4, format = (v: number) => String(v) } = $props();
</script>

<g class="axis-y">
  {#each ($yScale as ScaleLinear<number, number>).ticks(ticks) as tick (tick)}
    <g transform="translate(0, {$yScale(tick)})">
      <line
        x1={0}
        x2={$width}
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
