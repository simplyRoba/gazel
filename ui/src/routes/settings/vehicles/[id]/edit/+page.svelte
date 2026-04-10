<script lang="ts">
  import { page } from "$app/state";
  import { goto } from "$app/navigation";
  import { resolve } from "$app/paths";
  import { onMount } from "svelte";
  import type { Vehicle, CreateVehicle } from "$lib/api";
  import { fetchVehicle } from "$lib/api";
  import { t } from "$lib/i18n";
  import PageContainer from "$lib/components/PageContainer.svelte";
  import VehicleForm from "$lib/components/VehicleForm.svelte";
  import { updateVehicle, getError } from "$lib/stores/vehicles.svelte";

  let vehicle = $state<Vehicle | null>(null);
  let loadError = $state<string | null>(null);
  let saving = $state(false);

  onMount(async () => {
    const id = Number(page.params.id);
    if (isNaN(id)) {
      loadError = t("vehicle.edit.invalidId");
      return;
    }
    try {
      vehicle = await fetchVehicle(id);
    } catch {
      loadError = t("vehicle.edit.notFound");
    }
  });

  async function handleSave(data: CreateVehicle) {
    if (!vehicle) return;
    saving = true;
    const updated = await updateVehicle(vehicle.id, data);
    if (updated) {
      goto(resolve("/settings"));
    }
    saving = false;
  }
</script>

<PageContainer width="narrow">
  <h1 class="page-title">{t("vehicle.edit.title")}</h1>

  {#if loadError}
    <div class="error-banner">{loadError}</div>
  {:else if !vehicle}
    <p class="loading">{t("common.loading")}</p>
  {:else}
    {#if getError()}
      <div class="error-banner">{getError()}</div>
    {/if}

    <VehicleForm
      initial={vehicle}
      onsave={handleSave}
      oncancel={() => goto(resolve("/settings"))}
      {saving}
    />
  {/if}
</PageContainer>

<style>
  .page-title {
    font-size: var(--font-xl);
    font-weight: var(--font-weight-semibold);
    margin-bottom: var(--space-6);
  }

  .error-banner {
    padding: var(--space-3);
    margin-bottom: var(--space-4);
    background: var(--color-bg-sunken);
    color: var(--color-error);
    font-size: var(--font-sm);
    border: 1px solid var(--color-error);
  }

  .loading {
    color: var(--color-text-secondary);
    font-size: var(--font-sm);
  }
</style>
