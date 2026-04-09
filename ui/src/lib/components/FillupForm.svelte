<script lang="ts">
  import type { Fillup, CreateFillup } from "$lib/api";
  import { getSettings } from "$lib/stores/settings.svelte";
  import { getFillupsByVehicle } from "$lib/stores/fillups.svelte";

  const VOLUME_LABELS: Record<string, string> = { l: "L", gal: "gal" };
  const CURRENCY_SYMBOLS: Record<string, string> = { USD: "$", EUR: "\u20AC" };

  let {
    initial,
    vehicleId,
    odoMode = "total",
    onsave,
    oncancel,
    ondelete,
    saving = false,
  }: {
    initial?: Fillup;
    vehicleId: number;
    odoMode?: "total" | "trip";
    onsave: (data: CreateFillup) => Promise<void>;
    oncancel: () => void;
    ondelete?: () => void;
    saving?: boolean;
  } = $props();

  const settings = $derived(getSettings());
  const distanceUnit = $derived(settings.distance_unit);
  const volumeUnit = $derived(
    VOLUME_LABELS[settings.volume_unit] ?? settings.volume_unit,
  );
  const currencySymbol = $derived(
    CURRENCY_SYMBOLS[settings.currency] ?? settings.currency,
  );

  // Last odometer reading for min constraint (create mode only)
  const lastOdometer = $derived.by(() => {
    if (initial) return undefined; // no min constraint in edit mode
    const fillups = getFillupsByVehicle(vehicleId);
    if (fillups.length === 0) return undefined;
    return fillups[0].odometer;
  });

  // Track previous odoMode to convert values on switch
  let prevOdoMode = $state(odoMode);

  // Form fields — odometer defaults to last reading in create mode
  let date = $state(initial?.date ?? new Date().toISOString().slice(0, 10));
  let odometer = $state(
    initial?.odometer?.toString() ?? lastOdometer?.toString() ?? "",
  );

  // Convert odometer value when odoMode prop changes
  $effect(() => {
    if (odoMode === prevOdoMode) return;
    const currentVal = parseFloat(String(odometer));
    if (odoMode === "trip" && prevOdoMode === "total") {
      if (
        !isNaN(currentVal) &&
        lastOdometer != null &&
        currentVal >= lastOdometer
      ) {
        odometer = (currentVal - lastOdometer).toString();
      } else {
        odometer = "";
      }
    } else if (odoMode === "total" && prevOdoMode === "trip") {
      if (!isNaN(currentVal) && lastOdometer != null) {
        odometer = (lastOdometer + currentVal).toString();
      }
    }
    prevOdoMode = odoMode;
  });
  let fuelAmount = $state(initial?.fuel_amount?.toString() ?? "");
  let cost = $state(initial?.cost?.toString() ?? "");
  let station = $state(initial?.station ?? "");
  let notes = $state(initial?.notes ?? "");
  let isFullTank = $state(initial?.is_full_tank ?? true);
  let isMissed = $state(initial?.is_missed ?? false);

  // Resolve the absolute odometer value from either entry mode
  function resolveOdometer(): number {
    const raw = parseFloat(String(odometer));
    if (isNaN(raw)) return NaN;
    if (odoMode === "trip" && lastOdometer != null) {
      return lastOdometer + raw;
    }
    return raw;
  }

  // Validation errors
  let dateError = $state("");
  let odometerError = $state("");
  let fuelAmountError = $state("");
  let costError = $state("");

  // Smart missed fill-up prompt
  let showMissedPrompt = $state(false);

  // Re-fill when initial changes
  $effect(() => {
    if (initial) {
      date = initial.date;
      odometer = initial.odometer.toString();
      fuelAmount = initial.fuel_amount.toString();
      cost = initial.cost.toString();
      station = initial.station ?? "";
      notes = initial.notes ?? "";
      isFullTank = initial.is_full_tank;
      isMissed = initial.is_missed;
    }
  });

  // Check for suspicious odometer gap
  $effect(() => {
    if (initial) {
      showMissedPrompt = false;
      return;
    }

    const absoluteOdo = resolveOdometer();
    if (isNaN(absoluteOdo) || absoluteOdo <= 0) {
      showMissedPrompt = false;
      return;
    }

    const fillups = getFillupsByVehicle(vehicleId);
    if (fillups.length < 2) {
      showMissedPrompt = false;
      return;
    }

    const lastOdo = fillups[0]?.odometer ?? 0;
    if (lastOdo <= 0) {
      showMissedPrompt = false;
      return;
    }

    // Calculate average gap from existing fill-ups
    const odometers = fillups.map((f) => f.odometer).filter((o) => o > 0);
    if (odometers.length < 2) {
      showMissedPrompt = false;
      return;
    }

    let totalGap = 0;
    for (let i = 0; i < odometers.length - 1; i++) {
      totalGap += odometers[i] - odometers[i + 1];
    }
    const avgGap = totalGap / (odometers.length - 1);

    const currentGap = absoluteOdo - lastOdo;
    showMissedPrompt = avgGap > 0 && currentGap > avgGap * 1.75;
  });

  function handleSubmit(e: Event) {
    e.preventDefault();
    dateError = "";
    odometerError = "";
    fuelAmountError = "";
    costError = "";

    let valid = true;

    if (!date.trim()) {
      dateError = "Date is required.";
      valid = false;
    }

    const odoRaw = parseFloat(String(odometer));
    const odoVal = resolveOdometer();
    if (String(odometer).trim() === "" || isNaN(odoRaw)) {
      odometerError = "Odometer is required.";
      valid = false;
    } else if (odoMode === "trip" && odoRaw < 0) {
      odometerError = "Trip distance must be positive.";
      valid = false;
    } else if (odoMode === "total" && odoVal < 0) {
      odometerError = "Odometer must be positive.";
      valid = false;
    } else if (lastOdometer != null && odoVal < lastOdometer) {
      odometerError = `Must be at least ${lastOdometer} (last reading).`;
      valid = false;
    }

    const fuelVal = parseFloat(String(fuelAmount));
    if (String(fuelAmount).trim() === "" || isNaN(fuelVal)) {
      fuelAmountError = "Fuel amount is required.";
      valid = false;
    } else if (fuelVal <= 0) {
      fuelAmountError = "Fuel amount must be greater than zero.";
      valid = false;
    }

    const costVal = parseFloat(String(cost));
    if (String(cost).trim() === "" || isNaN(costVal)) {
      costError = "Cost is required.";
      valid = false;
    } else if (costVal < 0) {
      costError = "Cost must not be negative.";
      valid = false;
    }

    if (!valid) return;

    const resolvedOdo = resolveOdometer();
    const data: CreateFillup = {
      date: date.trim(),
      odometer: resolvedOdo,
      fuel_amount: fuelVal,
      cost: costVal,
      is_full_tank: isFullTank,
      is_missed: isMissed,
      station: station.trim() || null,
      notes: notes.trim() || null,
    };

    onsave(data);
  }
</script>

<form id="fillup-form" onsubmit={handleSubmit}>
  <div class="form-group">
    <label class="form-label" for="f-date">Date *</label>
    <div class="input-wrap" class:input-error={!!dateError}>
      <input
        id="f-date"
        class="input"
        type="date"
        bind:value={date}
        disabled={saving}
      />
    </div>
    {#if dateError}<span class="field-error">{dateError}</span>{/if}
  </div>

  <div class="form-group">
    <label class="form-label" for="f-odometer"
      >{odoMode === "trip" ? `Trip distance` : `Odometer`} ({distanceUnit}) *</label
    >
    <div class="input-wrap" class:input-error={!!odometerError}>
      <input
        id="f-odometer"
        class="input"
        type="number"
        step="any"
        min={odoMode === "trip" ? 0 : lastOdometer}
        bind:value={odometer}
        placeholder={odoMode === "trip" ? "e.g. 520" : "e.g. 45230"}
        disabled={saving}
      />
    </div>
    {#if odometerError}<span class="field-error">{odometerError}</span>{/if}
    {#if showMissedPrompt && !isMissed}
      <div class="missed-prompt">
        <span>That's a larger gap than usual. Did you miss a fill-up?</span>
        <button
          type="button"
          class="btn btn-ghost btn-sm"
          onclick={() => (isMissed = true)}
        >
          Yes, mark as missed
        </button>
      </div>
    {/if}
  </div>

  <div class="form-row">
    <div class="form-group">
      <label class="form-label" for="f-fuel">Fuel amount ({volumeUnit}) *</label
      >
      <div class="input-wrap" class:input-error={!!fuelAmountError}>
        <input
          id="f-fuel"
          class="input"
          type="number"
          step="any"
          bind:value={fuelAmount}
          placeholder="e.g. 42.3"
          disabled={saving}
        />
      </div>
      {#if fuelAmountError}<span class="field-error">{fuelAmountError}</span
        >{/if}
    </div>
    <div class="form-group">
      <label class="form-label" for="f-cost">Cost ({currencySymbol}) *</label>
      <div class="input-wrap" class:input-error={!!costError}>
        <input
          id="f-cost"
          class="input"
          type="number"
          step="any"
          bind:value={cost}
          placeholder="e.g. 78.50"
          disabled={saving}
        />
      </div>
      {#if costError}<span class="field-error">{costError}</span>{/if}
    </div>
  </div>

  <div class="form-group">
    <label class="form-label" for="f-station">Station</label>
    <div class="input-wrap">
      <input
        id="f-station"
        class="input"
        type="text"
        bind:value={station}
        placeholder="e.g. Shell Main St"
        disabled={saving}
      />
    </div>
  </div>

  <div class="form-group">
    <label class="form-label" for="f-notes">Notes</label>
    <div class="input-wrap">
      <textarea
        id="f-notes"
        class="input"
        bind:value={notes}
        placeholder="Optional"
        rows="2"
        disabled={saving}
      ></textarea>
    </div>
  </div>

  <div class="toggle-row">
    <label class="toggle-label">
      <input type="checkbox" bind:checked={isFullTank} disabled={saving} />
      <span>Full tank</span>
    </label>
    <label class="toggle-label">
      <input type="checkbox" bind:checked={isMissed} disabled={saving} />
      <span>Missed fill-up before this</span>
    </label>
  </div>

  <div class="form-actions">
    <button type="submit" class="btn btn-primary" disabled={saving}>
      {saving ? "Saving..." : initial ? "Save changes" : "Add fill-up"}
    </button>
    {#if initial && ondelete}
      <button
        type="button"
        class="btn btn-danger"
        disabled={saving}
        onclick={ondelete}
      >
        Delete
      </button>
    {/if}
    <button type="button" class="btn" disabled={saving} onclick={oncancel}>
      Cancel
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

  .toggle-row {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-4);
    margin-bottom: var(--space-4);
  }

  .toggle-label {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    font-size: var(--font-sm);
    color: var(--color-text-secondary);
    cursor: pointer;
  }

  .toggle-label input[type="checkbox"] {
    width: 18px;
    height: 18px;
    accent-color: var(--color-accent);
    cursor: pointer;
  }

  .missed-prompt {
    margin-top: var(--space-2);
    padding: var(--space-3);
    background: var(--color-bg-raised);
    border: 1px solid var(--color-warning);
    font-size: var(--font-xs);
    color: var(--color-text-secondary);
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .missed-prompt .btn {
    align-self: flex-start;
  }

  @media (max-width: 768px) {
    .form-row {
      grid-template-columns: 1fr;
    }
  }
</style>
