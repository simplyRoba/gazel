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
  import { getSettings } from "$lib/stores/settings.svelte";
  import { formatDistance, formatVolume, formatCurrency } from "$lib/format";
  import type { Fillup, CreateFillup } from "$lib/api";

  // Modal state
  let showFillupModal = $state(false);
  let editingFillup = $state<Fillup | undefined>(undefined);

  const settings = $derived(getSettings());

  onMount(async () => {
    await loadVehicles();
    const vehicles = getVehicles();
    if (vehicles.length > 0) {
      setActiveVehicle(vehicles[0].id);
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
  }

  async function handleDelete(fillupId: number) {
    const vehicleId = getActiveVehicleId();
    if (!vehicleId) return;
    await storeDeleteFillup(vehicleId, fillupId);
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
  {:else if getVehicles().length === 0}
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
    <!-- Vehicle chips -->
    <div class="vehicle-chips">
      {#each getVehicles() as vehicle (vehicle.id)}
        <button
          class="chip"
          class:active={getActiveVehicleId() === vehicle.id}
          onclick={() => handleChipClick(vehicle.id)}
        >
          {vehicle.name}
        </button>
      {/each}
    </div>

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
        {#each getFillups() as fillup (fillup.id)}
          <button
            class="card fillup-card"
            onclick={() => openEditModal(fillup)}
          >
            <div class="fillup-header">
              <span class="fillup-date">{formatDate(fillup.date)}</span>
              <span class="fillup-odometer mono"
                >{formatDistance(fillup.odometer, settings.distance_unit)}</span
              >
            </div>
            <div class="fillup-details">
              <span class="fillup-fuel mono"
                >{formatVolume(fillup.fuel_amount, settings.volume_unit)}</span
              >
              <span class="fillup-cost mono"
                >{formatCurrency(fillup.cost, settings.currency)}</span
              >
              {#if fillup.station}
                <span class="fillup-station">{fillup.station}</span>
              {/if}
            </div>
            <div class="fillup-badges">
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
  .vehicle-chips {
    display: flex;
    gap: var(--space-1);
    margin-bottom: var(--space-4);
    overflow-x: auto;
  }

  .actions-row {
    display: flex;
    justify-content: flex-end;
    margin-bottom: var(--space-4);
  }

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

  .fillup-odometer {
    font-size: var(--font-xs);
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
