<script lang="ts">
  import type { SegmentHistory } from "$lib/api";
  import { t } from "$lib/i18n";
  import DistanceChart from "./DistanceChart.svelte";
  import CostChart from "./CostChart.svelte";
  import FuelPriceChart from "./FuelPriceChart.svelte";

  let {
    segments,
    distanceUnit,
    volumeUnit,
    currency,
  }: {
    segments: SegmentHistory[];
    distanceUnit: string;
    volumeUnit: string;
    currency: string;
  } = $props();

  const hasData = $derived(segments.length >= 2);
</script>

{#if hasData}
  <div class="charts-stack">
    <CostChart {segments} {currency} />
    <DistanceChart {segments} {distanceUnit} />
    <FuelPriceChart {segments} {currency} {volumeUnit} />
  </div>
{:else}
  <div class="card charts-empty">
    <span class="charts-empty-text">{t("charts.empty")}</span>
  </div>
{/if}

<style>
  .charts-stack {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
    flex: 1;
    min-height: 0;
  }

  .charts-stack > :global(*) {
    flex: 1 0 auto;
  }

  .charts-empty {
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .charts-empty-text {
    font-size: var(--font-sm);
    color: var(--color-text-tertiary);
  }
</style>
