<script lang="ts">
  import type { Fillup, SegmentHistory } from "#lib/api.js";
  import DistanceChart from "./DistanceChart.svelte";
  import CostChart from "./CostChart.svelte";
  import FuelPriceChart from "./FuelPriceChart.svelte";

  let {
    segments,
    fillups,
    distanceUnit,
    volumeUnit,
    currency,
  }: {
    segments: SegmentHistory[];
    fillups: Fillup[];
    distanceUnit: string;
    volumeUnit: string;
    currency: string;
  } = $props();
</script>

<div class="charts-stack">
  <CostChart {segments} {currency} />
  <DistanceChart {segments} {distanceUnit} />
  <FuelPriceChart {fillups} {currency} {volumeUnit} />
</div>

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
</style>
