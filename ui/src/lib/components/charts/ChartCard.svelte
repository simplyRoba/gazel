<script lang="ts">
  import type { Snippet } from "svelte";
  import { t } from "$lib/i18n";
  import { LayerCake, Svg } from "layercake";
  import type { ScaleTime, ScaleBand, ScaleLinear, ScalePoint } from "d3-scale";

  /* eslint-disable @typescript-eslint/no-explicit-any */
  let {
    title,
    data,
    x,
    y,
    xScale = undefined,
    yDomain = undefined,
    padding = { top: 10, right: 10, bottom: 30, left: 52 },
    height = 180,
    actions,
    children,
  }: {
    title: string;
    data: Record<string, any>[];
    x: string | ((d: any) => any);
    y: string | ((d: any) => any);
    xScale?:
      | ScaleTime<number, number>
      | ScaleBand<string>
      | ScalePoint<string>
      | ScaleLinear<number, number>;
    yDomain?: [number | null, number | null];
    padding?: { top?: number; right?: number; bottom?: number; left?: number };
    height?: number;
    actions?: Snippet;
    children: Snippet;
  } = $props();
  /* eslint-enable @typescript-eslint/no-explicit-any */

  const hasEnoughData = $derived(data.length >= 2);
</script>

<div class="card chart-card">
  <div class="chart-header">
    <span class="chart-title">{title}</span>
    {#if actions}
      {@render actions()}
    {/if}
  </div>
  {#if hasEnoughData}
    <div class="chart-container" style="height: {height}px">
      <LayerCake {data} {x} {y} {xScale} {yDomain} {padding}>
        <Svg>
          {@render children()}
        </Svg>
      </LayerCake>
    </div>
  {:else}
    <div class="chart-empty" style="min-height: {height}px">
      <span class="chart-empty-text">{t("charts.needMore")}</span>
    </div>
  {/if}
</div>

<style>
  .chart-card {
    padding: var(--space-3);
    display: flex;
    flex-direction: column;
  }

  .chart-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-2);
    margin-bottom: var(--space-2);
  }

  .chart-title {
    font-size: var(--font-xs);
    font-weight: var(--font-weight-medium);
    color: var(--color-text-secondary);
  }

  .chart-container {
    position: relative;
    width: 100%;
    flex: 1 1 auto;
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
