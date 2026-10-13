<script lang="ts">
  import { getLayerCakeContext } from "layercake";
  import type { ScaleTime } from "d3-scale";
  import type { ChartScales } from "#lib/charts.js";
  import { getSettings } from "#lib/stores/settings.svelte.js";

  const chart = getLayerCakeContext<ChartScales>();

  const locale = $derived(getSettings().locale);

  let {
    ticks = 5,
    formatTick = (v: Date | string) => {
      if (v instanceof Date) {
        return v.toLocaleDateString(locale, {
          month: "short",
          year: "2-digit",
        });
      }
      // For band scale month labels like "2025-03", format nicely
      if (typeof v === "string" && /^\d{4}-\d{2}$/.test(v)) {
        const [y, m] = v.split("-");
        const d = new Date(Number(y), Number(m) - 1);
        return d.toLocaleDateString(locale, {
          month: "short",
          year: "2-digit",
        });
      }
      return String(v);
    },
  } = $props();
</script>

<g class="axis-x">
  {#if "ticks" in chart.xScale}
    {@const scale = chart.xScale as ScaleTime<number, number>}
    {#each scale.ticks(ticks) as tick (tick.getTime())}
      {@const x = scale(tick)}
      <g transform="translate({x}, {chart.height})">
        <line
          y1={0}
          y2={-chart.height}
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
  {:else}
    {@const scale = chart.xScale}
    {@const domain = scale.domain()}
    {@const bw = "bandwidth" in scale ? scale.bandwidth() : 0}
    {@const labelWidth = 48}
    {@const step = Math.max(
      1,
      Math.ceil((domain.length * labelWidth) / chart.width),
    )}
    {#each domain as tick, i (tick)}
      {@const x = scale(tick) ?? 0}
      <g transform="translate({x + bw / 2}, {chart.height})">
        {#if i % step === 0}
          <line y1={0} y2={4} stroke="var(--color-border-subtle)" />
          <text
            y={16}
            text-anchor="middle"
            fill="var(--color-text-tertiary)"
            font-size="11"
            font-family="var(--font-family)"
          >
            {formatTick(tick)}
          </text>
        {/if}
      </g>
    {/each}
  {/if}
  <line
    x1={0}
    x2={chart.width}
    y1={chart.height}
    y2={chart.height}
    stroke="var(--color-border-subtle)"
  />
</g>
