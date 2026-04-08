<script lang="ts">
  import type { Vehicle, CreateVehicle } from "$lib/api";

  const FUEL_TYPES = [
    { value: "gasoline", label: "Gasoline" },
    { value: "diesel", label: "Diesel" },
    { value: "electric", label: "Electric" },
    { value: "hybrid", label: "Hybrid" },
    { value: "lpg", label: "LPG" },
    { value: "cng", label: "CNG" },
    { value: "other", label: "Other" },
  ];

  let {
    initial,
    onsave,
    saving = false,
  }: {
    initial?: Vehicle;
    onsave: (data: CreateVehicle) => Promise<void>;
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
      nameError = "Vehicle name is required.";
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
    <label class="form-label" for="v-name">Name *</label>
    <div class="input-wrap" class:input-error={!!nameError}>
      <input
        id="v-name"
        class="input"
        type="text"
        bind:value={name}
        placeholder="e.g. Daily Driver"
        disabled={saving}
      />
    </div>
    {#if nameError}
      <span class="field-error">{nameError}</span>
    {/if}
  </div>

  <div class="form-row">
    <div class="form-group">
      <label class="form-label" for="v-make">Make</label>
      <div class="input-wrap">
        <input
          id="v-make"
          class="input"
          type="text"
          bind:value={make}
          placeholder="e.g. Honda"
          disabled={saving}
        />
      </div>
    </div>
    <div class="form-group">
      <label class="form-label" for="v-model">Model</label>
      <div class="input-wrap">
        <input
          id="v-model"
          class="input"
          type="text"
          bind:value={model}
          placeholder="e.g. Civic"
          disabled={saving}
        />
      </div>
    </div>
  </div>

  <div class="form-row">
    <div class="form-group">
      <label class="form-label" for="v-year">Year</label>
      <div class="input-wrap">
        <input
          id="v-year"
          class="input"
          type="number"
          bind:value={year}
          placeholder="e.g. 2024"
          min="1900"
          max="2100"
          disabled={saving}
        />
      </div>
    </div>
    <div class="form-group">
      <label class="form-label" for="v-fuel">Fuel type</label>
      <div class="input-wrap">
        <select
          id="v-fuel"
          class="input"
          bind:value={fuelType}
          disabled={saving}
        >
          {#each FUEL_TYPES as ft (ft.value)}
            <option value={ft.value}>{ft.label}</option>
          {/each}
        </select>
      </div>
    </div>
  </div>

  <div class="form-group">
    <label class="form-label" for="v-notes">Notes</label>
    <div class="input-wrap">
      <textarea
        id="v-notes"
        class="input"
        bind:value={notes}
        placeholder="Optional"
        rows="3"
        disabled={saving}
      ></textarea>
    </div>
  </div>

  <div class="form-actions">
    <button type="submit" class="btn btn-primary btn-full" disabled={saving}>
      {saving ? "Saving..." : initial ? "Save changes" : "Add vehicle"}
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
