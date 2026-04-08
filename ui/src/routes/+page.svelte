<script lang="ts">
  import { onMount } from "svelte";
  import { resolve } from "$app/paths";
  import { Car } from "lucide-svelte";
  import PageContainer from "$lib/components/PageContainer.svelte";
  import EmptyState from "$lib/components/EmptyState.svelte";
  import {
    loadVehicles,
    getVehicles,
    getLoading,
  } from "$lib/stores/vehicles.svelte";

  onMount(() => {
    loadVehicles();
  });
</script>

<PageContainer width="wide">
  {#if getLoading()}
    <p class="loading">Loading...</p>
  {:else if getVehicles().length === 0}
    <EmptyState
      icon={Car}
      heading="No vehicles yet"
      description="Add your first vehicle to start tracking fill-ups and fuel efficiency."
    >
      {#snippet action()}
        <a href={resolve("/settings/vehicles/new")} class="btn btn-primary">
          Add vehicle
        </a>
      {/snippet}
    </EmptyState>
  {:else}
    <!-- Vehicle chips -->
    <div class="vehicle-chips">
      {#each getVehicles() as vehicle, i (vehicle.id)}
        <button class="chip" class:active={i === 0}>
          {vehicle.name}
        </button>
      {/each}
    </div>

    <p class="placeholder-text">
      Dashboard content coming soon. Manage vehicles in
      <a href={resolve("/settings")} class="link">Settings</a>.
    </p>
  {/if}
</PageContainer>

<style>
  .loading {
    color: var(--color-text-secondary);
    font-size: var(--font-sm);
    padding: var(--space-8);
    text-align: center;
  }

  .vehicle-chips {
    display: flex;
    gap: var(--space-1);
    margin-bottom: var(--space-6);
    overflow-x: auto;
  }

  .placeholder-text {
    color: var(--color-text-secondary);
    font-size: var(--font-sm);
    text-align: center;
    padding: var(--space-8);
  }

  .link {
    color: var(--color-accent-text);
    text-decoration: underline;
  }
</style>
