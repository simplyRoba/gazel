<script lang="ts">
  import type { Vehicle, CreateVehicle } from "$lib/api";
  import { t } from "$lib/i18n";

  const FUEL_TYPES = [
    { value: "gasoline", key: "vehicle.fuel.gasoline" },
    { value: "diesel", key: "vehicle.fuel.diesel" },
    { value: "electric", key: "vehicle.fuel.electric" },
    { value: "hybrid", key: "vehicle.fuel.hybrid" },
    { value: "lpg", key: "vehicle.fuel.lpg" },
    { value: "cng", key: "vehicle.fuel.cng" },
    { value: "other", key: "vehicle.fuel.other" },
  ];

  let {
    initial,
    onsave,
    oncancel,
    saving = false,
  }: {
    initial?: Vehicle;
    onsave: (data: CreateVehicle) => Promise<void>;
    oncancel: () => void;
    saving?: boolean;
  } = $props();

  let name = $state(initial?.name ?? "");
  let make = $state(initial?.make ?? "");
  let model = $state(initial?.model ?? "");
  let year = $state(initial?.year?.toString() ?? "");
  let fuelType = $state(initial?.fuel_type ?? "gasoline");
  let notes = $state(initial?.notes ?? "");
  let nameError = $state("");

  // Re-fill when initial changes (edit page load)
  $effect(() => {
    if (initial) {
      name = initial.name;
      make = initial.make ?? "";
      model = initial.model ?? "";
      year = initial.year?.toString() ?? "";
      fuelType = initial.fuel_type;
      notes = initial.notes ?? "";
    }
  });

  function handleSubmit(e: Event) {
    e.preventDefault();
    nameError = "";

    if (!name.trim()) {
      nameError = t("vehicle.form.nameRequired");
      return;
    }

    const data: CreateVehicle = {
      name: name.trim(),
      make: make.trim() || null,
      model: model.trim() || null,
      year: year ? parseInt(year, 10) : null,
      fuel_type: fuelType,
      notes: notes.trim() || null,
    };

    onsave(data);
  }
</script>

<form id="vehicle-form" onsubmit={handleSubmit}>
  <div class="form-group">
    <label class="form-label" for="v-name">{t("vehicle.form.name")}</label>
    <div class="input-wrap" class:input-error={!!nameError}>
      <input
        id="v-name"
        class="input"
        type="text"
        bind:value={name}
        placeholder={t("vehicle.form.namePlaceholder")}
        disabled={saving}
      />
    </div>
    {#if nameError}
      <span class="field-error">{nameError}</span>
    {/if}
  </div>

  <div class="form-row">
    <div class="form-group">
      <label class="form-label" for="v-make">{t("vehicle.form.make")}</label>
      <div class="input-wrap">
        <input
          id="v-make"
          class="input"
          type="text"
          bind:value={make}
          placeholder={t("vehicle.form.makePlaceholder")}
          disabled={saving}
        />
      </div>
    </div>
    <div class="form-group">
      <label class="form-label" for="v-model">{t("vehicle.form.model")}</label>
      <div class="input-wrap">
        <input
          id="v-model"
          class="input"
          type="text"
          bind:value={model}
          placeholder={t("vehicle.form.modelPlaceholder")}
          disabled={saving}
        />
      </div>
    </div>
  </div>

  <div class="form-row">
    <div class="form-group">
      <label class="form-label" for="v-year">{t("vehicle.form.year")}</label>
      <div class="input-wrap">
        <input
          id="v-year"
          class="input"
          type="number"
          bind:value={year}
          placeholder={t("vehicle.form.yearPlaceholder")}
          min="1900"
          max="2100"
          disabled={saving}
        />
      </div>
    </div>
    <div class="form-group">
      <label class="form-label" for="v-fuel">{t("vehicle.form.fuelType")}</label
      >
      <div class="input-wrap">
        <select
          id="v-fuel"
          class="input"
          bind:value={fuelType}
          disabled={saving}
        >
          {#each FUEL_TYPES as ft (ft.value)}
            <option value={ft.value}>{t(ft.key)}</option>
          {/each}
        </select>
      </div>
    </div>
  </div>

  <div class="form-group">
    <label class="form-label" for="v-notes">{t("vehicle.form.notes")}</label>
    <div class="input-wrap">
      <textarea
        id="v-notes"
        class="input"
        bind:value={notes}
        placeholder={t("common.optional")}
        rows="3"
        disabled={saving}
      ></textarea>
    </div>
  </div>

  <div class="form-actions">
    <button type="submit" class="btn btn-primary" disabled={saving}>
      {saving
        ? t("common.saving")
        : initial
          ? t("vehicle.form.saveChanges")
          : t("vehicle.form.addVehicle")}
    </button>
    <button type="button" class="btn" disabled={saving} onclick={oncancel}>
      {t("common.cancel")}
    </button>
  </div>
</form>

<style>
  .form-group {
    margin-bottom: var(--space-4);
  }

  .form-label {
    display: block;
    font-size: var(--font-sm);
    font-weight: var(--font-weight-medium);
    color: var(--color-text-secondary);
    margin-bottom: var(--space-1);
  }

  .form-row {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: var(--space-3);
  }

  .form-actions {
    display: flex;
    gap: var(--space-3);
    margin-top: var(--space-6);
  }

  .field-error {
    font-size: var(--font-xs);
    color: var(--color-error);
    margin-top: var(--space-1);
    display: block;
  }

  @media (max-width: 768px) {
    .form-row {
      grid-template-columns: 1fr;
    }
  }
</style>
