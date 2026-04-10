<script lang="ts">
  import { onMount } from "svelte";
  import { resolve } from "$app/paths";
  import { Car, Fuel } from "lucide-svelte";
  import PageContainer from "$lib/components/PageContainer.svelte";
  import EmptyState from "$lib/components/EmptyState.svelte";
  import FillupModal from "$lib/components/FillupModal.svelte";
  import ChartsPanel from "$lib/components/charts/ChartsPanel.svelte";
  import EfficiencyChart from "$lib/components/charts/EfficiencyChart.svelte";
  import MonthlyCostChart from "$lib/components/charts/MonthlyCostChart.svelte";
  import FuelPriceChart from "$lib/components/charts/FuelPriceChart.svelte";
  import Sparkline from "$lib/components/charts/Sparkline.svelte";
  import {
    loadVehicles,
    getVehicles,
    getLoading as getVehiclesLoading,
  } from "$lib/stores/vehicles.svelte";
  import {
    getFillups,
    getLoading as getFillupsLoading,
    getActiveVehicleId,
    setActiveVehicle,
    createFillup as storeCreateFillup,
    updateFillup as storeUpdateFillup,
    deleteFillup as storeDeleteFillup,
  } from "$lib/stores/fillups.svelte";
  import {
    getVehicleStats,
    getVehicleHistory,
    getLoading as getStatsLoading,
    loadAllStats,
    invalidateStats,
  } from "$lib/stores/stats.svelte";
  import { getSettings } from "$lib/stores/settings.svelte";
  import {
    formatDistance,
    formatCurrency,
    formatVolume,
    formatEfficiency,
  } from "$lib/format";
  import type { Fillup, CreateFillup } from "$lib/api";
  import {
    buildEfficiencyMap,
    getEfficiencyForFillup,
    computeFleetSummary,
  } from "$lib/stats";
  import { toSparklineData } from "$lib/charts";

  // Modal state
  let showFillupModal = $state(false);
  let editingFillup = $state<Fillup | undefined>(undefined);

  const settings = $derived(getSettings());

  // ── Fleet summary (derived from per-vehicle stats) ─────

  const vehicles = $derived(getVehicles());

  const fleetSummary = $derived(computeFleetSummary(vehicles, getVehicleStats));

  // ── Per-vehicle stats for active vehicle ───────────────

  const activeStats = $derived.by(() => {
    const id = getActiveVehicleId();
    if (id === null) return null;
    return getVehicleStats(id) ?? null;
  });

  const activeHistory = $derived.by(() => {
    const id = getActiveVehicleId();
    if (id === null) return [];
    return getVehicleHistory(id);
  });

  // ── Sparkline data for summary cards ──────────────────

  const costPerDistanceSparkline = $derived(
    activeHistory.length >= 2
      ? toSparklineData(activeHistory, (s) => s.cost_per_distance)
      : [],
  );

  const fuelPriceSparkline = $derived(
    activeHistory.length >= 2
      ? toSparklineData(
          activeHistory.filter((s) => s.fuel > 0),
          (s) => s.cost / s.fuel,
        )
      : [],
  );

  // ── Segment-to-fillup efficiency map ───────────────────

  const efficiencyMap = $derived(buildEfficiencyMap(activeHistory));

  // ── Mobile carousel state ─────────────────────────────

  let activeChartIndex = $state(0);
  let carouselEl = $state<HTMLDivElement | null>(null);

  function handleCarouselScroll() {
    if (!carouselEl) return;
    const scrollLeft = carouselEl.scrollLeft;
    const cardWidth = carouselEl.offsetWidth;
    activeChartIndex = Math.round(scrollLeft / cardWidth);
  }

  // ── Lifecycle ──────────────────────────────────────────

  onMount(async () => {
    await loadVehicles();
    const vs = getVehicles();
    if (vs.length > 0) {
      setActiveVehicle(vs[0].id);
      loadAllStats(vs.map((v) => v.id));
    }
  });

  function handleChipClick(vehicleId: number) {
    setActiveVehicle(vehicleId);
  }

  function openCreateModal() {
    editingFillup = undefined;
    showFillupModal = true;
  }

  function openEditModal(fillup: Fillup) {
    editingFillup = fillup;
    showFillupModal = true;
  }

  function closeModal() {
    showFillupModal = false;
    editingFillup = undefined;
  }

  async function handleSave(data: CreateFillup) {
    const vehicleId = getActiveVehicleId();
    if (!vehicleId) return;

    let result;
    if (editingFillup) {
      result = await storeUpdateFillup(vehicleId, editingFillup.id, data);
    } else {
      result = await storeCreateFillup(vehicleId, data);
    }
    if (!result) {
      throw new Error("Save failed");
    }
    invalidateStats(vehicleId);
  }

  async function handleDelete(fillupId: number) {
    const vehicleId = getActiveVehicleId();
    if (!vehicleId) return;
    const ok = await storeDeleteFillup(vehicleId, fillupId);
    if (ok) {
      invalidateStats(vehicleId);
    }
  }

  function formatDate(dateStr: string): string {
    const d = new Date(dateStr + "T00:00:00");
    return d.toLocaleDateString(undefined, {
      month: "short",
      day: "numeric",
      year: "numeric",
    });
  }
</script>

<PageContainer width="wide">
  {#if getVehiclesLoading()}
    <div class="skeleton-area">
      <div class="shimmer chip-skeleton"></div>
      <div class="shimmer chip-skeleton"></div>
    </div>
  {:else if vehicles.length === 0}
    <EmptyState
      icon={Car}
      heading="No vehicles yet"
      description="Add your first vehicle to start tracking fill-ups and fuel efficiency."
    >
      {#snippet action()}
        <a href={resolve("/settings/vehicles/new")} class="btn btn-primary"
          >Add vehicle</a
        >
      {/snippet}
    </EmptyState>
  {:else}
    <!-- Summary cards (always shown — single vehicle or aggregated) -->
    {#if getStatsLoading() && !fleetSummary}
      <div class="summary-grid" data-testid="summary-cards-loading">
        {#each Array(4) as _, i (i)}
          <div class="card shimmer summary-skeleton"></div>
        {/each}
      </div>
    {:else if fleetSummary}
      <div class="summary-grid" data-testid="summary-cards">
        <div class="card summary-card">
          <span class="summary-value mono"
            >{formatDistance(
              fleetSummary.totalDistance,
              settings.distance_unit,
            )}</span
          >
          <span class="summary-label">Total distance</span>
        </div>
        <div class="card summary-card">
          <span class="summary-value">{fleetSummary.totalFillups}</span>
          <span class="summary-label">Fill-ups</span>
        </div>
        <div class="card summary-card summary-card--sparkline">
          <span class="summary-value mono">
            {#if fleetSummary.costPerDistance !== null}
              {formatCurrency(
                fleetSummary.costPerDistance,
                settings.currency,
              )}/{settings.distance_unit}
            {:else}
              &mdash;
            {/if}
          </span>
          <span class="summary-label">Cost per {settings.distance_unit}</span>
          {#if costPerDistanceSparkline.length >= 2}
            <div class="sparkline-bg">
              <Sparkline data={costPerDistanceSparkline} />
            </div>
          {/if}
        </div>
        <div class="card summary-card summary-card--sparkline">
          <span class="summary-value mono">
            {#if fleetSummary.costPerVolume !== null}
              {formatCurrency(
                fleetSummary.costPerVolume,
                settings.currency,
              )}/{settings.volume_unit === "l" ? "L" : settings.volume_unit}
            {:else}
              &mdash;
            {/if}
          </span>
          <span class="summary-label">Fuel price</span>
          {#if fuelPriceSparkline.length >= 2}
            <div class="sparkline-bg">
              <Sparkline data={fuelPriceSparkline} />
            </div>
          {/if}
        </div>
      </div>
    {/if}

    {#if vehicles.length > 1}
      <!-- Vehicle chips + per-vehicle stats (multi-vehicle only) -->
      <div class="vehicle-chips">
        {#each vehicles as vehicle (vehicle.id)}
          <button
            class="chip"
            class:active={getActiveVehicleId() === vehicle.id}
            onclick={() => handleChipClick(vehicle.id)}
          >
            {vehicle.name}
          </button>
        {/each}
      </div>

      {#if getStatsLoading() && !activeStats}
        <div class="vehicle-stats-row" data-testid="vehicle-stats-loading">
          <div class="shimmer vehicle-stat-skeleton"></div>
          <div class="shimmer vehicle-stat-skeleton"></div>
          <div class="shimmer vehicle-stat-skeleton"></div>
          <div class="shimmer vehicle-stat-skeleton"></div>
        </div>
      {:else if activeStats}
        <div class="vehicle-stats-row" data-testid="vehicle-stats">
          <div class="vehicle-stat">
            <span class="vehicle-stat-value mono"
              >{formatDistance(
                activeStats.total_distance,
                settings.distance_unit,
              )}</span
            >
            <span class="vehicle-stat-label">Total distance</span>
          </div>
          <div class="vehicle-stat">
            <span class="vehicle-stat-value">{activeStats.fill_up_count}</span>
            <span class="vehicle-stat-label">Fill-ups</span>
          </div>
          <div class="vehicle-stat">
            <span class="vehicle-stat-value mono">
              {#if activeStats.average_cost_per_distance !== null}
                {formatCurrency(
                  activeStats.average_cost_per_distance,
                  settings.currency,
                )}/{settings.distance_unit}
              {:else}
                &mdash;
              {/if}
            </span>
            <span class="vehicle-stat-label"
              >Cost per {settings.distance_unit}</span
            >
          </div>
          <div class="vehicle-stat">
            <span class="vehicle-stat-value mono">
              {#if activeStats.total_fuel > 0}
                {formatCurrency(
                  activeStats.total_cost / activeStats.total_fuel,
                  settings.currency,
                )}/{settings.volume_unit === "l" ? "L" : settings.volume_unit}
              {:else}
                &mdash;
              {/if}
            </span>
            <span class="vehicle-stat-label">Fuel price</span>
          </div>
        </div>
      {/if}
    {/if}

    <!-- ── Mobile chart carousel (<=768px) ────────────── -->
    {#if getStatsLoading() && activeHistory.length === 0}
      <div class="chart-carousel-wrapper">
        <div class="card skeleton-chart">
          <div
            class="shimmer"
            style="width: 60px; height: 10px; margin-bottom: var(--space-2)"
          ></div>
          <div class="shimmer" style="width: 100%; height: 150px"></div>
        </div>
      </div>
    {:else if activeHistory.length >= 2}
      <div class="chart-carousel-wrapper">
        <div
          class="chart-carousel"
          bind:this={carouselEl}
          onscroll={handleCarouselScroll}
        >
          <div class="chart-carousel-item">
            <EfficiencyChart
              segments={activeHistory}
              distanceUnit={settings.distance_unit}
              volumeUnit={settings.volume_unit}
            />
          </div>
          <div class="chart-carousel-item">
            <MonthlyCostChart
              segments={activeHistory}
              currency={settings.currency}
            />
          </div>
          <div class="chart-carousel-item">
            <FuelPriceChart
              segments={activeHistory}
              currency={settings.currency}
              volumeUnit={settings.volume_unit}
            />
          </div>
        </div>
        <div class="carousel-dots">
          {#each [0, 1, 2] as idx (idx)}
            <span class="carousel-dot" class:active={activeChartIndex === idx}
            ></span>
          {/each}
        </div>
      </div>
    {/if}

    <!-- ── Two-column layout (desktop: charts + fill-ups) ── -->
    <div class="dashboard-content">
      <!-- Charts panel (desktop/tablet only, sticky) -->
      {#if getStatsLoading() && activeHistory.length === 0}
        <div class="charts-column">
          {#each Array(3) as _, i (i)}
            <div class="card skeleton-chart">
              <div
                class="shimmer"
                style="width: 60px; height: 10px; margin-bottom: var(--space-2)"
              ></div>
              <div class="shimmer" style="width: 100%; height: 150px"></div>
            </div>
          {/each}
        </div>
      {:else if activeHistory.length >= 2}
        <div class="charts-column">
          <ChartsPanel
            segments={activeHistory}
            distanceUnit={settings.distance_unit}
            volumeUnit={settings.volume_unit}
            currency={settings.currency}
          />
        </div>
      {/if}

      <!-- Fill-up list column -->
      <div class="fillups-column">
        <!-- Fill-up list -->
        {#if getFillupsLoading()}
          <div class="fillup-list">
            {#each Array(3) as _, i (i)}
              <div class="card skeleton-fillup">
                <div class="skeleton-fillup-header">
                  <div class="shimmer" style="width: 90px; height: 12px"></div>
                  <div class="shimmer" style="width: 60px; height: 10px"></div>
                </div>
                <div class="skeleton-fillup-details">
                  <div class="shimmer" style="width: 50px; height: 12px"></div>
                  <div class="shimmer" style="width: 45px; height: 12px"></div>
                  <div class="shimmer" style="width: 70px; height: 12px"></div>
                </div>
                <div class="skeleton-fillup-badges">
                  <div
                    class="shimmer"
                    style="width: 55px; height: 18px; border-radius: 9px"
                  ></div>
                </div>
              </div>
            {/each}
          </div>
        {:else if getFillups().length === 0}
          <EmptyState
            icon={Fuel}
            heading="No fill-ups yet"
            description="Record your first fill-up for this vehicle."
          >
            {#snippet action()}
              <button class="btn btn-primary" onclick={openCreateModal}
                >Add fill-up</button
              >
            {/snippet}
          </EmptyState>
        {:else}
          <div class="fillup-list">
            {#each getFillups() as fillup, i (fillup.id)}
              {@const eff = getEfficiencyForFillup(fillup, efficiencyMap)}
              {@const fillups = getFillups()}
              {@const prev = i < fillups.length - 1 ? fillups[i + 1] : null}
              {@const odoDiff = prev ? fillup.odometer - prev.odometer : null}
              {@const fuelPrice =
                fillup.fuel_amount > 0
                  ? fillup.cost / fillup.fuel_amount
                  : null}
              <button
                class="card fillup-card"
                onclick={() => openEditModal(fillup)}
              >
                <div class="fillup-header">
                  <span class="fillup-date">{formatDate(fillup.date)}</span>
                  <span class="fillup-odo">
                    <span class="fillup-diff mono"
                      >{#if odoDiff !== null}+{formatDistance(
                          odoDiff,
                          settings.distance_unit,
                        )}{:else}&mdash;{/if}</span
                    >
                    <span class="fillup-abs mono"
                      >{fillup.odometer.toLocaleString()}</span
                    >
                  </span>
                </div>
                <div class="fillup-details">
                  <span class="fillup-fuel mono"
                    >{formatVolume(
                      fillup.fuel_amount,
                      settings.volume_unit,
                    )}</span
                  >
                  <span class="fillup-cost mono"
                    >{formatCurrency(fillup.cost, settings.currency)}</span
                  >
                  {#if fuelPrice !== null}
                    <span class="fillup-price mono"
                      >{formatCurrency(
                        fuelPrice,
                        settings.currency,
                      )}/{settings.volume_unit === "l"
                        ? "L"
                        : settings.volume_unit}</span
                    >
                  {/if}
                  {#if fillup.station}
                    <span class="fillup-station">{fillup.station}</span>
                  {/if}
                </div>
                <div class="fillup-badges">
                  {#if eff !== null}
                    <span class="badge" data-testid="efficiency-badge"
                      >{formatEfficiency(
                        eff,
                        settings.distance_unit,
                        settings.volume_unit,
                      )}</span
                    >
                  {/if}
                  {#if fillup.is_full_tank}
                    <span class="badge badge-success">Full tank</span>
                  {/if}
                  {#if fillup.is_missed}
                    <span class="badge badge-warning">Missed</span>
                  {/if}
                </div>
              </button>
            {/each}
          </div>
        {/if}
      </div>
    </div>
  {/if}
</PageContainer>

{#if getActiveVehicleId()}
  <FillupModal
    open={showFillupModal}
    vehicleId={getActiveVehicleId()!}
    initial={editingFillup}
    onsave={handleSave}
    ondelete={handleDelete}
    onclose={closeModal}
  />
{/if}

<style>
  /* ── Summary cards grid ─────────────────────────────── */
  .summary-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(140px, 1fr));
    gap: var(--space-2);
    margin-bottom: var(--space-4);
  }

  .summary-card {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
    padding: var(--space-3);
    background: var(--color-bg-feature);
    border-color: var(--color-border-feature);
  }

  .summary-card--sparkline {
    position: relative;
    overflow: hidden;
  }

  .summary-value {
    font-size: var(--font-md);
    font-weight: var(--font-weight-semibold);
    color: var(--color-accent-text);
    line-height: var(--line-height-tight);
    position: relative;
    z-index: 1;
  }

  .summary-label {
    font-size: var(--font-xs);
    color: var(--color-text-secondary);
    position: relative;
    z-index: 1;
  }

  .sparkline-bg {
    position: absolute;
    bottom: 0;
    left: 0;
    right: 0;
    height: 32px;
    opacity: 0.4;
  }

  .summary-skeleton {
    height: 56px;
  }

  /* ── Vehicle chips ──────────────────────────────────── */
  .vehicle-chips {
    display: flex;
    gap: var(--space-1);
    margin-bottom: var(--space-3);
    overflow-x: auto;
  }

  /* ── Per-vehicle stats row ──────────────────────────── */
  .vehicle-stats-row {
    display: flex;
    gap: var(--space-6);
    margin-bottom: var(--space-4);
    padding: var(--space-3) 0;
    border-bottom: 1px solid var(--color-border-subtle);
  }

  .vehicle-stat {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
  }

  .vehicle-stat-value {
    font-size: var(--font-sm);
    font-weight: var(--font-weight-semibold);
    color: var(--color-text);
  }

  .vehicle-stat-label {
    font-size: var(--font-xs);
    color: var(--color-text-secondary);
  }

  .vehicle-stat-skeleton {
    width: 100px;
    height: 36px;
  }

  /* ── Mobile chart carousel (shown <=768px) ──────────── */
  .chart-carousel-wrapper {
    display: block;
    margin-bottom: var(--space-4);
  }

  .chart-carousel {
    display: flex;
    overflow-x: auto;
    scroll-snap-type: x mandatory;
    -webkit-overflow-scrolling: touch;
    scrollbar-width: none;
    gap: var(--space-3);
  }

  .chart-carousel::-webkit-scrollbar {
    display: none;
  }

  .chart-carousel-item {
    flex: 0 0 100%;
    scroll-snap-align: start;
  }

  .carousel-dots {
    display: flex;
    justify-content: center;
    gap: var(--space-2);
    margin-top: var(--space-2);
  }

  .carousel-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--color-border);
    transition: background var(--transition-fast);
  }

  .carousel-dot.active {
    background: var(--color-accent);
  }

  /* ── Two-column dashboard content ──────────────────── */
  .dashboard-content {
    display: flex;
    flex-direction: column;
  }

  .charts-column {
    display: none;
  }

  .fillups-column {
    flex: 1;
    min-width: 0;
  }

  /* Desktop/tablet: side-by-side layout.
     Breakpoint at 960px ensures charts column gets >=320px
     (enough for axis labels). Below 960px the mobile carousel
     + full-width list provides a better experience. */
  @media (min-width: 960px) {
    .chart-carousel-wrapper {
      display: none;
    }

    .dashboard-content:has(.charts-column) {
      display: grid;
      grid-template-columns: 2fr 3fr;
      gap: var(--space-4);
      align-items: start;
    }

    .charts-column {
      display: block;
      position: sticky;
      top: var(--space-4);
    }
  }

  /* Widescreen: give charts more room */
  @media (min-width: 1280px) {
    .dashboard-content:has(.charts-column) {
      grid-template-columns: 1fr 1fr;
    }
  }

  /* ── Fill-up list ───────────────────────────────────── */
  .fillup-list {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
  }

  .fillup-card {
    display: block;
    width: 100%;
    text-align: left;
    cursor: pointer;
    padding: var(--space-4);
    background: var(--color-bg-raised);
    border: 1px solid var(--color-border);
    transition: border-color var(--transition-fast);
  }

  .fillup-card:hover {
    border-color: var(--color-accent);
  }

  .fillup-header {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
    margin-bottom: var(--space-2);
  }

  .fillup-date {
    font-weight: var(--font-weight-medium);
    font-size: var(--font-sm);
  }

  .fillup-odo {
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    gap: 1px;
  }

  .fillup-diff {
    font-size: var(--font-xs);
    color: var(--color-text-secondary);
  }

  .fillup-abs {
    font-size: 10px;
    color: var(--color-text-tertiary);
  }

  .fillup-price {
    color: var(--color-text-secondary);
  }

  .fillup-details {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-3);
    font-size: var(--font-sm);
    margin-bottom: var(--space-2);
  }

  .fillup-fuel,
  .fillup-cost {
    font-weight: var(--font-weight-medium);
  }

  .fillup-station {
    color: var(--color-text-secondary);
  }

  .fillup-badges {
    display: flex;
    gap: var(--space-2);
  }

  .fillup-badges:empty {
    display: none;
  }

  /* ── Skeletons ──────────────────────────────────────── */
  .skeleton-area {
    display: flex;
    gap: var(--space-1);
    margin-bottom: var(--space-6);
  }

  .chip-skeleton {
    width: 80px;
    height: 36px;
  }

  .skeleton-fillup {
    padding: var(--space-4);
  }

  .skeleton-fillup-header {
    display: flex;
    justify-content: space-between;
    margin-bottom: var(--space-3);
  }

  .skeleton-fillup-details {
    display: flex;
    gap: var(--space-3);
    margin-bottom: var(--space-3);
  }

  .skeleton-fillup-badges {
    display: flex;
    gap: var(--space-2);
  }

  .skeleton-chart {
    padding: var(--space-3);
  }

  .skeleton-chart + .skeleton-chart {
    margin-top: var(--space-3);
  }
</style>
