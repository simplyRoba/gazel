<script lang="ts">
  import { onMount } from "svelte";
  import { resolve } from "$app/paths";
  import { Car, Fuel, Plus } from "lucide-svelte";
  import PageContainer from "$lib/components/PageContainer.svelte";
  import EmptyState from "$lib/components/EmptyState.svelte";
  import FillupModal from "$lib/components/FillupModal.svelte";
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
    formatVolume,
    formatCurrency,
    formatEfficiency,
  } from "$lib/format";
  import type { Fillup, CreateFillup } from "$lib/api";
  import {
    buildEfficiencyMap,
    getEfficiencyForFillup,
    computeFleetSummary,
  } from "$lib/stats";

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

  // ── Segment-to-fillup efficiency map ───────────────────

  const efficiencyMap = $derived(buildEfficiencyMap(activeHistory));

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
        <div class="card summary-card">
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
        </div>
        <div class="card summary-card">
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

    <!-- Add fill-up button -->
    <div class="actions-row">
      <button class="btn btn-primary btn-sm" onclick={openCreateModal}>
        <Plus size={16} />
        Add fill-up
      </button>
    </div>

    <!-- Fill-up list -->
    {#if getFillupsLoading()}
      <div class="fillup-list">
        {#each Array(3) as _, i (i)}
          <div class="card shimmer skeleton-card"></div>
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
            fillup.fuel_amount > 0 ? fillup.cost / fillup.fuel_amount : null}
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
                >{formatVolume(fillup.fuel_amount, settings.volume_unit)}</span
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

  .summary-value {
    font-size: var(--font-md);
    font-weight: var(--font-weight-semibold);
    color: var(--color-accent-text);
    line-height: var(--line-height-tight);
  }

  .summary-label {
    font-size: var(--font-xs);
    color: var(--color-text-secondary);
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

  /* ── Actions row ────────────────────────────────────── */
  .actions-row {
    display: flex;
    justify-content: flex-end;
    margin-bottom: var(--space-4);
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

  .skeleton-card {
    height: 80px;
  }
</style>
