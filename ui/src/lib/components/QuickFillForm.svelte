<script lang="ts">
  import type { CreateFillup } from "$lib/api";
  import { t } from "$lib/i18n";
  import { getSettings } from "$lib/stores/settings.svelte";
  import { getFillupsByVehicle } from "$lib/stores/fillups.svelte";
  import {
    deriveFuelPriceTotal,
    formatDecimalInput,
    formatEfficiency,
    parseDecimal,
    type FuelPriceTotalField,
  } from "$lib/format";
  import {
    guardNumericBeforeInput,
    normalizeOnBlur,
    recordFieldEdit,
  } from "$lib/numeric-input";

  const VOLUME_LABELS: Record<string, string> = { l: "L", gal: "gal" };
  const CURRENCY_SYMBOLS: Record<string, string> = { USD: "$", EUR: "\u20AC" };

  let {
    vehicleId,
    odoMode = "total",
    onsave,
    oncancel,
    saving = false,
  }: {
    vehicleId: number;
    odoMode?: "total" | "trip";
    onsave: (data: CreateFillup) => Promise<void>;
    oncancel: () => void;
    saving?: boolean;
  } = $props();

  const settings = $derived(getSettings());
  const locale = $derived(settings.locale);
  const distanceUnit = $derived(settings.distance_unit);
  const volumeUnit = $derived(
    VOLUME_LABELS[settings.volume_unit] ?? settings.volume_unit,
  );
  const currencySymbol = $derived(
    CURRENCY_SYMBOLS[settings.currency] ?? settings.currency,
  );

  // Last odometer reading for prefill + min constraint
  const lastOdometer = $derived.by(() => {
    const fillups = getFillupsByVehicle(vehicleId);
    if (fillups.length === 0) return undefined;
    return fillups[0].odometer;
  });

  // Track previous odoMode to convert the odometer value on switch
  let prevOdoMode = $state<"total" | "trip">("total");

  // Form fields (strings — parsed via parseDecimal on use)
  let date = $state(new Date().toISOString().slice(0, 10));
  let odometer = $state("");
  let fuelAmount = $state("");
  let pricePerUnit = $state("");
  let total = $state("");
  let station = $state("");
  let notes = $state("");
  let isFullTank = $state(true);
  let isMissed = $state(false);

  let showDetails = $state(false);

  // Validation errors
  let odometerError = $state("");
  let fuelAmountError = $state("");
  let totalError = $state("");
  let dateError = $state("");

  // Tracks whether the user has edited the odometer field, so a late/background
  // store refresh of `lastOdometer` cannot clobber what they typed.
  let odometerTouched = $state(false);

  // Prefill odometer from last reading. Depends only on `lastOdometer` — it
  // must NOT react to `odoMode`, otherwise toggling total/trip would re-prefill
  // the absolute odometer and clobber the conversion effect below. Once the
  // user has edited the field, stop prefilling.
  $effect(() => {
    if (!odometerTouched && lastOdometer != null) {
      odometer = lastOdometer.toString();
    }
  });

  // Convert odometer when odoMode changes (mirrors FillupForm behavior)
  $effect(() => {
    if (odoMode === prevOdoMode) return;
    const currentVal = parseDecimal(odometer, locale);
    if (odoMode === "trip" && prevOdoMode === "total") {
      if (
        !isNaN(currentVal) &&
        lastOdometer != null &&
        currentVal > lastOdometer
      ) {
        odometer = (currentVal - lastOdometer).toString();
      } else {
        odometer = "";
      }
    } else if (odoMode === "total" && prevOdoMode === "trip") {
      if (lastOdometer != null) {
        odometer = (
          lastOdometer + (isNaN(currentVal) ? 0 : currentVal)
        ).toString();
      } else if (isNaN(currentVal)) {
        odometer = "";
      }
    }
    prevOdoMode = odoMode;
  });

  // ── Fuel / price / total auto-calc ──────────────────────
  // Track the two most-recently-edited money fields; the third is derived.
  let recentEdits = $state<FuelPriceTotalField[]>(["fuel", "total"]);

  function recompute(justEdited: FuelPriceTotalField) {
    recentEdits = recordFieldEdit(recentEdits, justEdited);
    const authoritative = recentEdits.slice(0, 2) as [
      FuelPriceTotalField,
      FuelPriceTotalField,
    ];
    const fields = {
      fuel: parseDecimal(fuelAmount, locale),
      price: parseDecimal(pricePerUnit, locale),
      total: parseDecimal(total, locale),
    };
    const result = deriveFuelPriceTotal(
      {
        fuel: Number.isNaN(fields.fuel) ? undefined : fields.fuel,
        price: Number.isNaN(fields.price) ? undefined : fields.price,
        total: Number.isNaN(fields.total) ? undefined : fields.total,
      },
      authoritative,
    );
    if (!result) return;
    // Never overwrite the field currently being edited.
    if (result.field === justEdited) return;
    const formatted = formatDecimalInput(
      result.value,
      locale,
      result.field === "fuel" ? 2 : result.field === "total" ? 2 : 3,
    );
    if (result.field === "fuel") fuelAmount = formatted;
    else if (result.field === "price") pricePerUnit = formatted;
    else total = formatted;
  }

  // Resolve absolute odometer value from total/trip mode
  function resolveOdometer(): number {
    const raw = parseDecimal(odometer, locale);
    if (isNaN(raw)) return NaN;
    if (odoMode === "trip" && lastOdometer != null) {
      return lastOdometer + raw;
    }
    return raw;
  }

  // ── Live efficiency preview ─────────────────────────────
  const efficiencyPreview = $derived.by(() => {
    const fuel = parseDecimal(fuelAmount, locale);
    if (isNaN(fuel) || fuel <= 0) return null;
    if (lastOdometer == null) return null;
    const absoluteOdo = resolveOdometer();
    if (isNaN(absoluteOdo)) return null;
    const distance = absoluteOdo - lastOdometer;
    if (distance <= 0) return null;
    const kmPerVolume = distance / fuel;
    return formatEfficiency(
      kmPerVolume,
      settings.distance_unit,
      settings.volume_unit,
      locale,
    );
  });

  // ── Smart missed fill-up prompt (mirrors FillupForm) ────
  const showMissedPrompt = $derived.by(() => {
    const absoluteOdo = resolveOdometer();
    if (isNaN(absoluteOdo) || absoluteOdo <= 0) return false;
    const fillups = getFillupsByVehicle(vehicleId);
    if (fillups.length < 2) return false;
    const lastOdo = fillups[0]?.odometer ?? 0;
    if (lastOdo <= 0) return false;
    const odometers = fillups.map((f) => f.odometer).filter((o) => o > 0);
    if (odometers.length < 2) return false;
    let totalGap = 0;
    for (let i = 0; i < odometers.length - 1; i++) {
      totalGap += odometers[i] - odometers[i + 1];
    }
    const avgGap = totalGap / (odometers.length - 1);
    const currentGap = absoluteOdo - lastOdo;
    return avgGap > 0 && currentGap > avgGap * 1.75;
  });

  // ── Input handlers ──────────────────────────────────────
  const guardNumeric = guardNumericBeforeInput(false);
  const guardOdometer = $derived(guardNumericBeforeInput(odoMode === "trip"));

  function blurField(
    setter: (v: string) => void,
    value: string,
    decimals: number,
  ) {
    setter(normalizeOnBlur(value, locale, decimals));
  }

  function handleSubmit(e: Event) {
    e.preventDefault();
    odometerError = "";
    fuelAmountError = "";
    totalError = "";
    dateError = "";

    let valid = true;

    if (!date.trim()) {
      dateError = t("fillup.form.dateRequired");
      valid = false;
    }

    const odoRaw = parseDecimal(odometer, locale);
    const odoVal = resolveOdometer();
    if (String(odometer).trim() === "" || isNaN(odoRaw)) {
      odometerError = t("fillup.form.odometerRequired");
      valid = false;
    } else if (odoMode === "trip" && odoRaw < 0) {
      odometerError = t("fillup.form.tripPositive");
      valid = false;
    } else if (odoMode === "total" && odoVal < 0) {
      odometerError = t("fillup.form.odometerPositive");
      valid = false;
    } else if (lastOdometer != null && odoVal < lastOdometer) {
      odometerError = t("fillup.form.odometerMin", { min: lastOdometer });
      valid = false;
    }

    const fuelVal = parseDecimal(fuelAmount, locale);
    if (String(fuelAmount).trim() === "" || isNaN(fuelVal)) {
      fuelAmountError = t("fillup.form.fuelRequired");
      valid = false;
    } else if (fuelVal <= 0) {
      fuelAmountError = t("fillup.form.fuelPositive");
      valid = false;
    }

    const totalVal = parseDecimal(total, locale);
    if (String(total).trim() === "" || isNaN(totalVal)) {
      totalError = t("fillup.form.costRequired");
      valid = false;
    } else if (totalVal < 0) {
      totalError = t("fillup.form.costPositive");
      valid = false;
    }

    if (!valid) return;

    const data: CreateFillup = {
      date: date.trim(),
      odometer: resolveOdometer(),
      fuel_amount: fuelVal,
      cost: totalVal,
      is_full_tank: isFullTank,
      is_missed: isMissed,
      station: station.trim() || null,
      notes: notes.trim() || null,
    };

    onsave(data);
  }
</script>

<form id="quick-fill-form" onsubmit={handleSubmit}>
  <div class="form-group">
    <label class="form-label" for="q-odometer"
      >{odoMode === "trip"
        ? t("fillup.form.tripDistance")
        : t("fillup.form.odometer")} ({distanceUnit}) *</label
    >
    <div class="input-wrap" class:input-error={!!odometerError}>
      <input
        id="q-odometer"
        class="input input-lg num"
        type="text"
        inputmode="decimal"
        autocomplete="off"
        bind:value={odometer}
        oninput={() => (odometerTouched = true)}
        onbeforeinput={guardOdometer}
        onblur={() => blurField((v) => (odometer = v), odometer, 1)}
        placeholder={odoMode === "trip"
          ? t("fillup.form.tripPlaceholder")
          : t("fillup.form.odometerPlaceholder")}
        disabled={saving}
      />
    </div>
    {#if odometerError}<span class="field-error">{odometerError}</span>{/if}
    {#if showMissedPrompt && !isMissed}
      <div class="missed-prompt">
        <span>{t("fillup.missed.prompt")}</span>
        <button
          type="button"
          class="btn btn-ghost btn-sm"
          onclick={() => (isMissed = true)}
        >
          {t("fillup.missed.confirm")}
        </button>
      </div>
    {/if}
  </div>

  <div class="money-row">
    <div class="form-group">
      <label class="form-label" for="q-fuel"
        >{t("fillup.form.fuelAmount")} ({volumeUnit}) *</label
      >
      <div class="input-wrap" class:input-error={!!fuelAmountError}>
        <input
          id="q-fuel"
          class="input input-lg num"
          type="text"
          inputmode="decimal"
          autocomplete="off"
          bind:value={fuelAmount}
          oninput={() => recompute("fuel")}
          onbeforeinput={guardNumeric}
          onblur={() => blurField((v) => (fuelAmount = v), fuelAmount, 2)}
          placeholder={t("fillup.form.fuelPlaceholder")}
          disabled={saving}
        />
      </div>
      {#if fuelAmountError}<span class="field-error">{fuelAmountError}</span
        >{/if}
    </div>

    <div class="form-group">
      <label class="form-label" for="q-price"
        >{t("fillup.quick.pricePerUnit", { unit: volumeUnit })} ({currencySymbol})</label
      >
      <div class="input-wrap">
        <input
          id="q-price"
          class="input input-lg num"
          type="text"
          inputmode="decimal"
          autocomplete="off"
          bind:value={pricePerUnit}
          oninput={() => recompute("price")}
          onbeforeinput={guardNumeric}
          onblur={() => blurField((v) => (pricePerUnit = v), pricePerUnit, 3)}
          placeholder={t("fillup.quick.pricePlaceholder")}
          disabled={saving}
        />
      </div>
    </div>

    <div class="form-group">
      <label class="form-label" for="q-total"
        >{t("fillup.quick.total")} ({currencySymbol}) *</label
      >
      <div class="input-wrap" class:input-error={!!totalError}>
        <input
          id="q-total"
          class="input input-lg num"
          type="text"
          inputmode="decimal"
          autocomplete="off"
          bind:value={total}
          oninput={() => recompute("total")}
          onbeforeinput={guardNumeric}
          onblur={() => blurField((v) => (total = v), total, 2)}
          placeholder={t("fillup.form.costPlaceholder")}
          disabled={saving}
        />
      </div>
      {#if totalError}<span class="field-error">{totalError}</span>{/if}
    </div>
  </div>

  <p class="auto-calc-hint">{t("fillup.quick.autoCalcHint")}</p>

  {#if efficiencyPreview}
    <div class="efficiency-preview num">
      {t("fillup.quick.efficiencyPreview", { value: efficiencyPreview })}
    </div>
  {/if}

  <button
    type="button"
    class="details-toggle"
    aria-expanded={showDetails}
    onclick={() => (showDetails = !showDetails)}
    disabled={saving}
  >
    {showDetails
      ? t("fillup.quick.lessDetails")
      : t("fillup.quick.moreDetails")}
  </button>

  {#if showDetails}
    <div class="details">
      <div class="form-group">
        <label class="form-label" for="q-date">{t("fillup.form.date")}</label>
        <div class="input-wrap" class:input-error={!!dateError}>
          <input
            id="q-date"
            class="input"
            type="date"
            bind:value={date}
            disabled={saving}
          />
        </div>
        {#if dateError}<span class="field-error">{dateError}</span>{/if}
      </div>

      <div class="form-group">
        <label class="form-label" for="q-station"
          >{t("fillup.form.station")}</label
        >
        <div class="input-wrap">
          <input
            id="q-station"
            class="input"
            type="text"
            bind:value={station}
            placeholder={t("fillup.form.stationPlaceholder")}
            disabled={saving}
          />
        </div>
      </div>

      <div class="form-group">
        <label class="form-label" for="q-notes">{t("fillup.form.notes")}</label>
        <div class="input-wrap">
          <textarea
            id="q-notes"
            class="input"
            bind:value={notes}
            placeholder={t("common.optional")}
            rows="2"
            disabled={saving}
          ></textarea>
        </div>
      </div>

      <div class="toggle-row">
        <label class="toggle-label">
          <input type="checkbox" bind:checked={isFullTank} disabled={saving} />
          <span>{t("fillup.form.fullTank")}</span>
        </label>
        <label class="toggle-label">
          <input type="checkbox" bind:checked={isMissed} disabled={saving} />
          <span>{t("fillup.form.missedBefore")}</span>
        </label>
      </div>
    </div>
  {/if}

  <div class="form-actions">
    <button type="submit" class="btn btn-primary btn-block" disabled={saving}>
      {saving ? t("common.saving") : t("fillup.form.addFillup")}
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

  /* Inputs are normal size on desktop; enlarged only on touch/mobile for
     thumb-friendly targets (see the media query below). */

  .num {
    font-family: var(--font-family-mono);
  }

  .money-row {
    display: grid;
    grid-template-columns: 1fr 1fr 1fr;
    gap: var(--space-3);
  }

  .auto-calc-hint {
    font-size: var(--font-xs);
    color: var(--color-text-tertiary);
    margin: calc(-1 * var(--space-2)) 0 var(--space-4);
  }

  .efficiency-preview {
    background: var(--color-bg-feature);
    border: 1px solid var(--color-border-feature);
    color: var(--color-accent-text);
    font-weight: var(--font-stat-weight);
    padding: var(--space-3) var(--space-4);
    margin-bottom: var(--space-4);
    text-align: center;
  }

  .details-toggle {
    background: none;
    border: none;
    color: var(--color-accent);
    font-size: var(--font-sm);
    font-weight: var(--font-weight-medium);
    cursor: pointer;
    padding: var(--space-2) 0;
    margin-bottom: var(--space-2);
  }

  .details-toggle:hover {
    color: var(--color-accent-hover);
  }

  .details {
    border-top: 1px solid var(--color-border-subtle);
    padding-top: var(--space-4);
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

  .form-actions {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
    margin-top: var(--space-6);
  }

  .btn-block {
    width: 100%;
  }

  @media (max-width: 768px) {
    .money-row {
      grid-template-columns: 1fr;
    }

    .input-lg {
      font-size: var(--font-xl);
      padding: var(--space-3) var(--space-4);
    }
  }
</style>
