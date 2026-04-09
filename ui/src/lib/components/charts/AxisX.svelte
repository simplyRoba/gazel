<script lang="ts">
  import { getContext } from "svelte";
  import type { LayerCakeContext } from "$lib/charts";

  const { xScale, height, width } = getContext("LayerCake") as LayerCakeContext;

  let {
    ticks = 5,
    formatTick = (v: Date | string) => {
      if (v instanceof Date) {
        return v.toLocaleDateString("en", { month: "short", year: "2-digit" });
      }
      // For band scale month labels like "2025-03", format nicely
      if (typeof v === "string" && /^\d{4}-\d{2}$/.test(v)) {
        const [y, m] = v.split("-");
        const d = new Date(Number(y), Number(m) - 1);
        return d.toLocaleDateString("en", { month: "short", year: "2-digit" });
      }
      return String(v);
    },
  } = $props();
</script>

<g class="axis-x">
  {#if "ticks" in $xScale}
    {#each ($xScale as { ticks: (n: number) => Date[] }).ticks(ticks) as tick (tick.getTime())}
      {@const x = ($xScale as (v: Date) => number)(tick)}
      <g transform="translate({x}, {$height})">
        <line
          y1={0}
          y2={-$height}
          stroke="var(--color-border-subtle)"
          stroke-dasharray="2,4"
          opacity="0.5"
        />
        <text
          y={16}
          text-anchor="middle"
          fill="var(--color-text-tertiary)"
          font-size="11"
          font-family="var(--font-family)"
        >
          {formatTick(tick)}
        </text>
      </g>
    {/each}
  {:else if "domain" in $xScale}
    {#each ($xScale as { domain: () => string[] }).domain() as tick (tick)}
      {@const x = ($xScale as (v: string) => number)(tick)}
      {@const bw =
        "bandwidth" in $xScale
          ? ($xScale as { bandwidth: () => number }).bandwidth()
          : 0}
      <g transform="translate({x + bw / 2}, {$height})">
        <text
          y={16}
          text-anchor="middle"
          fill="var(--color-text-tertiary)"
          font-size="11"
          font-family="var(--font-family)"
        >
          {formatTick(tick)}
        </text>
      </g>
    {/each}
  {/if}
  <line
    x1={0}
    x2={$width}
    y1={$height}
    y2={$height}
    stroke="var(--color-border-subtle)"
  />
</g>
