<script lang="ts">
  import type { Snippet } from "svelte";
  import { LayerCake, Svg } from "layercake";
  import type { ScaleTime, ScaleBand, ScaleLinear } from "d3-scale";

  /* eslint-disable @typescript-eslint/no-explicit-any */
  let {
    title,
    data,
    x,
    y,
    xScale = undefined,
    yDomain = undefined,
    padding = { top: 10, right: 10, bottom: 30, left: 44 },
    height = 180,
    children,
  }: {
    title: string;
    data: Record<string, any>[];
    x: string | ((d: any) => any);
    y: string | ((d: any) => any);
    xScale?:
      | ScaleTime<number, number>
      | ScaleBand<string>
      | ScaleLinear<number, number>;
    yDomain?: [number | null, number | null];
    padding?: { top?: number; right?: number; bottom?: number; left?: number };
    height?: number;
    children: Snippet;
  } = $props();
  /* eslint-enable @typescript-eslint/no-explicit-any */

  const hasEnoughData = $derived(data.length >= 2);
</script>

<div class="card chart-card">
  <span class="chart-title">{title}</span>
  {#if hasEnoughData}
    <div class="chart-container" style="height: {height}px">
      <LayerCake {data} {x} {y} {xScale} {yDomain} {padding}>
        <Svg>
          {@render children()}
        </Svg>
      </LayerCake>
    </div>
  {:else}
    <div class="chart-empty" style="height: {height}px">
      <span class="chart-empty-text">More fill-ups needed for chart</span>
    </div>
  {/if}
</div>

<style>
  .chart-card {
    padding: var(--space-3);
  }

  .chart-title {
    display: block;
    font-size: var(--font-xs);
    font-weight: var(--font-weight-medium);
    color: var(--color-text-secondary);
    margin-bottom: var(--space-2);
  }

  .chart-container {
    width: 100%;
  }

  .chart-empty {
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .chart-empty-text {
    font-size: var(--font-xs);
    color: var(--color-text-tertiary);
  }
</style>
