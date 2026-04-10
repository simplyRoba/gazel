<script lang="ts">
  import { goto } from "$app/navigation";
  import { resolve } from "$app/paths";
  import type { CreateVehicle } from "$lib/api";
  import { t } from "$lib/i18n";
  import PageContainer from "$lib/components/PageContainer.svelte";
  import VehicleForm from "$lib/components/VehicleForm.svelte";
  import { createVehicle, getError } from "$lib/stores/vehicles.svelte";

  let saving = $state(false);

  async function handleSave(data: CreateVehicle) {
    saving = true;
    const vehicle = await createVehicle(data);
    if (vehicle) {
      goto(resolve("/settings"));
    }
    saving = false;
  }
</script>

<PageContainer width="narrow">
  <h1 class="page-title">{t("vehicle.add.title")}</h1>

  {#if getError()}
    <div class="error-banner">{getError()}</div>
  {/if}

  <VehicleForm
    onsave={handleSave}
    oncancel={() => goto(resolve("/settings"))}
    {saving}
  />
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
</style>
