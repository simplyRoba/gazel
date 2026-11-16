<script lang="ts">
  import type { SelectedStats } from "#lib/api.js";
  import type { MonthlySelection } from "#lib/monthly-window.js";
  import DistanceChart from "./DistanceChart.svelte";
  import CostChart from "./CostChart.svelte";
  import FuelPriceChart from "./FuelPriceChart.svelte";

  let {
    charts,
    distanceUnit,
    volumeUnit,
    currency,
    vehicleId = null,
    costSelection = $bindable(null),
    distanceSelection = $bindable(null),
  }: {
    charts: SelectedStats["charts"] | null;
    distanceUnit: string;
    volumeUnit: string;
    currency: string;
    vehicleId?: number | null;
    costSelection?: MonthlySelection | null;
    distanceSelection?: MonthlySelection | null;
  } = $props();
</script>

<div class="charts-stack">
  <CostChart
    series={charts?.cost ?? null}
    {currency}
    {vehicleId}
    bind:selection={costSelection}
  />
  <DistanceChart
    series={charts?.distance ?? null}
    {distanceUnit}
    {vehicleId}
    bind:selection={distanceSelection}
  />
  <FuelPriceChart series={charts?.fuel_price ?? null} {currency} {volumeUnit} />
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
