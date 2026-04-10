<script lang="ts">
  import type { SegmentHistory } from "$lib/api";
  import EfficiencyChart from "./EfficiencyChart.svelte";
  import MonthlyCostChart from "./MonthlyCostChart.svelte";
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
    <EfficiencyChart {segments} {distanceUnit} {volumeUnit} />
    <MonthlyCostChart {segments} {currency} />
    <FuelPriceChart {segments} {currency} {volumeUnit} />
  </div>
{:else}
  <div class="card charts-empty">
    <span class="charts-empty-text">Add more fill-ups to see trends</span>
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
    flex: 1;
    min-height: 0;
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
