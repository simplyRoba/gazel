<script lang="ts">
  import { onMount } from "svelte";
  import { resolve } from "$app/paths";
  import { Pencil, Trash2, Car, Plus } from "lucide-svelte";
  import PageContainer from "$lib/components/PageContainer.svelte";
  import EmptyState from "$lib/components/EmptyState.svelte";
  import ModalDialog from "$lib/components/ModalDialog.svelte";
  import type { Vehicle } from "$lib/api";
  import {
    loadVehicles,
    getVehicles,
    getLoading,
    deleteVehicle,
  } from "$lib/stores/vehicles.svelte";

  let deleteTarget = $state<Vehicle | null>(null);

  onMount(() => {
    loadVehicles();
  });

  async function handleDeleteConfirm() {
    if (!deleteTarget) return;
    await deleteVehicle(deleteTarget.id);
    deleteTarget = null;
  }
</script>

<PageContainer>
  <h1 class="page-title">Settings</h1>
  <p class="page-subtitle">Configure vehicles, units, appearance, and data.</p>

  <!-- Vehicles section -->
  <section class="section vehicles-section">
    <h2 class="section-title">
      <Car size={14} />
      Vehicles
    </h2>

    {#if getLoading()}
      <div class="vehicle-list">
        <div class="skeleton-card vehicle-row">
          <div class="vehicle-info">
            <div class="shimmer" style="width:120px;height:14px"></div>
            <div
              class="shimmer"
              style="width:180px;height:10px;margin-top:6px"
            ></div>
          </div>
          <div class="row-actions">
            <div class="shimmer" style="width:40px;height:40px"></div>
            <div class="shimmer" style="width:40px;height:40px"></div>
          </div>
        </div>
      </div>
    {:else if getVehicles().length === 0}
      <EmptyState
        icon={Car}
        heading="No vehicles yet"
        description="Add your first vehicle to start tracking fill-ups."
      >
        {#snippet action()}
          <a href={resolve("/settings/vehicles/new")} class="btn btn-primary">
            <Plus size={16} />
            Add vehicle
          </a>
        {/snippet}
      </EmptyState>
    {:else}
      <div class="vehicle-list">
        {#each getVehicles() as vehicle (vehicle.id)}
          <div class="card vehicle-row corner-tri corner-tri-sm">
            <div class="vehicle-info">
              <span class="vehicle-name">{vehicle.name}</span>
              <span class="vehicle-meta">
                {[vehicle.make, vehicle.model, vehicle.year]
                  .filter(Boolean)
                  .join(" · ") || vehicle.fuel_type}
              </span>
            </div>
            <div class="row-actions">
              <a
                href={resolve("/settings/vehicles/[id]/edit", {
                  id: String(vehicle.id),
                })}
                class="btn btn-icon"
              >
                <Pencil size={16} />
              </a>
              <button
                class="btn btn-icon"
                onclick={() => (deleteTarget = vehicle)}
              >
                <Trash2 size={16} />
              </button>
            </div>
          </div>
        {/each}
      </div>

      <div class="add-action">
        <a href={resolve("/settings/vehicles/new")} class="btn btn-primary">
          <Plus size={16} />
          Add vehicle
        </a>
      </div>
    {/if}
  </section>
</PageContainer>

<ModalDialog
  open={!!deleteTarget}
  title="Delete vehicle"
  message={deleteTarget
    ? `Delete "${deleteTarget.name}"? This cannot be undone.`
    : ""}
  mode="confirm"
  variant="danger"
  confirmLabel="Delete"
  onconfirm={handleDeleteConfirm}
  oncancel={() => (deleteTarget = null)}
/>

<style>
  .page-title {
    font-size: var(--font-2xl);
    font-weight: var(--font-weight-bold);
    margin-bottom: var(--space-2);
  }

  .page-subtitle {
    font-size: var(--font-md);
    color: var(--color-text-secondary);
    margin-bottom: var(--space-8);
  }

  .vehicles-section {
    margin-bottom: var(--space-8);
  }

  .vehicle-list {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .vehicle-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--space-3) var(--space-4);
  }

  .vehicle-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .vehicle-name {
    font-size: var(--font-sm);
    font-weight: var(--font-weight-medium);
  }

  .vehicle-meta {
    font-size: var(--font-xs);
    color: var(--color-text-tertiary);
  }

  .row-actions {
    display: flex;
    gap: var(--space-2);
    align-items: center;
  }

  .add-action {
    margin-top: var(--space-4);
  }
</style>
