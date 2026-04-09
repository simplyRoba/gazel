<script lang="ts">
  import { onMount } from "svelte";
  import { resolve } from "$app/paths";
  import {
    Pencil,
    Trash2,
    Car,
    Plus,
    Sun,
    Moon,
    Monitor,
    Ruler,
  } from "lucide-svelte";
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
  import {
    getSettings,
    updateSettingsStore,
  } from "$lib/stores/settings.svelte";
  import {
    setTheme,
    getThemePreference,
    type ThemePreference,
  } from "$lib/stores/theme.svelte";

  let deleteTarget = $state<Vehicle | null>(null);

  onMount(() => {
    loadVehicles();
  });

  async function handleDeleteConfirm() {
    if (!deleteTarget) return;
    await deleteVehicle(deleteTarget.id);
    deleteTarget = null;
  }

  function handleTheme(pref: ThemePreference) {
    setTheme(pref);
  }

  async function handleUnitSystem(system: string) {
    if (system === "metric") {
      await updateSettingsStore({
        unit_system: "metric",
        distance_unit: "km",
        volume_unit: "l",
      });
    } else if (system === "imperial") {
      await updateSettingsStore({
        unit_system: "imperial",
        distance_unit: "mi",
        volume_unit: "gal",
      });
    } else {
      await updateSettingsStore({ unit_system: "custom" });
    }
  }

  async function handleDistanceUnit(unit: string) {
    await updateSettingsStore({ distance_unit: unit });
  }

  async function handleVolumeUnit(unit: string) {
    await updateSettingsStore({ volume_unit: unit });
  }

  async function handleCurrency(code: string) {
    await updateSettingsStore({ currency: code });
  }

  async function handleLocale(locale: string) {
    await updateSettingsStore({ locale });
  }

  const currencies = [
    { code: "USD", label: "USD" },
    { code: "EUR", label: "EUR" },
  ];
</script>

<PageContainer>
  <h1 class="page-title">Settings</h1>
  <p class="page-subtitle">Configure vehicles, units, appearance, and data.</p>

  <div class="settings-grid">
    <!-- Display -->
    <section class="section corner-tri corner-tri-sm">
      <h2 class="section-title">
        <Sun size={14} />
        Display
      </h2>

      <div class="setting-row">
        <span class="setting-label">Theme</span>
        <div class="chip-group">
          <button
            class="setting-chip"
            class:active={getThemePreference() === "light"}
            onclick={() => handleTheme("light")}
          >
            <Sun size={14} />
            Light
          </button>
          <button
            class="setting-chip"
            class:active={getThemePreference() === "dark"}
            onclick={() => handleTheme("dark")}
          >
            <Moon size={14} />
            Dark
          </button>
          <button
            class="setting-chip"
            class:active={getThemePreference() === "system"}
            onclick={() => handleTheme("system")}
          >
            <Monitor size={14} />
            System
          </button>
        </div>
      </div>

      <div class="setting-row">
        <span class="setting-label">Language</span>
        <div class="chip-group">
          <button
            class="setting-chip"
            class:active={getSettings().locale === "en"}
            onclick={() => handleLocale("en")}
          >
            English
          </button>
        </div>
      </div>
    </section>

    <!-- Units -->
    <section class="section corner-tri corner-tri-sm">
      <h2 class="section-title">
        <Ruler size={14} />
        Units
      </h2>

      <div class="setting-row">
        <span class="setting-label">System</span>
        <div class="chip-group">
          <button
            class="setting-chip"
            class:active={getSettings().unit_system === "metric"}
            onclick={() => handleUnitSystem("metric")}
          >
            Metric
          </button>
          <button
            class="setting-chip"
            class:active={getSettings().unit_system === "imperial"}
            onclick={() => handleUnitSystem("imperial")}
          >
            Imperial
          </button>
          <button
            class="setting-chip"
            class:active={getSettings().unit_system === "custom"}
            onclick={() => handleUnitSystem("custom")}
          >
            Custom
          </button>
        </div>
      </div>

      {#if getSettings().unit_system === "custom"}
        <div class="setting-row">
          <span class="setting-label">Distance</span>
          <div class="chip-group">
            <button
              class="setting-chip"
              class:active={getSettings().distance_unit === "km"}
              onclick={() => handleDistanceUnit("km")}
            >
              Kilometers
            </button>
            <button
              class="setting-chip"
              class:active={getSettings().distance_unit === "mi"}
              onclick={() => handleDistanceUnit("mi")}
            >
              Miles
            </button>
          </div>
        </div>

        <div class="setting-row">
          <span class="setting-label">Volume</span>
          <div class="chip-group">
            <button
              class="setting-chip"
              class:active={getSettings().volume_unit === "l"}
              onclick={() => handleVolumeUnit("l")}
            >
              Liters
            </button>
            <button
              class="setting-chip"
              class:active={getSettings().volume_unit === "gal"}
              onclick={() => handleVolumeUnit("gal")}
            >
              Gallons
            </button>
          </div>
        </div>
      {/if}

      <div class="setting-row">
        <span class="setting-label">Currency</span>
        <div class="chip-group">
          {#each currencies as c (c.code)}
            <button
              class="setting-chip"
              class:active={getSettings().currency === c.code}
              onclick={() => handleCurrency(c.code)}
            >
              {c.label}
            </button>
          {/each}
        </div>
      </div>
    </section>

    <!-- Vehicles -->
    <section class="section corner-tri corner-tri-sm">
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
  </div>
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

  /* ── Settings grid ─────────────────────────────── */

  .settings-grid {
    display: flex;
    flex-direction: column;
    gap: var(--space-6);
  }

  /* ── Setting rows ──────────────────────────────── */

  .setting-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--space-3) 0;
  }

  .setting-row + .setting-row {
    border-top: 1px solid var(--color-border-subtle);
  }

  .setting-label {
    font-size: var(--font-sm);
    font-weight: var(--font-weight-medium);
    color: var(--color-text);
    flex-shrink: 0;
    margin-right: var(--space-4);
  }

  /* ── Segmented control ──────────────────────────── */
  /* Chips connect into a single bar with shared borders. */

  .chip-group {
    display: inline-flex;
    border: 1px solid var(--color-border);
    background: var(--color-bg-sunken);
  }

  .setting-chip {
    display: inline-flex;
    align-items: center;
    gap: var(--space-1);
    padding: var(--space-2) var(--space-3);
    border: none;
    border-right: 1px solid var(--color-border);
    background: transparent;
    color: var(--color-text-secondary);
    font-size: var(--font-sm);
    font-weight: var(--font-weight-medium);
    font-family: inherit;
    line-height: 1.2;
    white-space: nowrap;
    cursor: pointer;
    box-sizing: border-box;
    position: relative;
    overflow: hidden;
    transition:
      color var(--transition-fast),
      background var(--transition-fast);
  }

  .setting-chip::after {
    content: "";
    position: absolute;
    top: 0;
    right: 0;
    width: 0;
    height: 0;
    border-style: solid;
    border-width: 0;
    border-color: transparent var(--color-accent) transparent transparent;
    transition: border-width var(--transition-fast);
  }

  .setting-chip:last-child {
    border-right: none;
  }

  .setting-chip:hover {
    color: var(--color-text);
    background: var(--color-bg);
  }

  .setting-chip:hover::after {
    border-width: 0 var(--corner-tri-sm) var(--corner-tri-sm) 0;
  }

  .setting-chip.active {
    color: var(--color-accent-text);
    background: var(--color-bg-raised);
    font-weight: var(--font-weight-semibold);
  }

  .setting-chip.active::after {
    border-width: 0 var(--corner-tri-sm) var(--corner-tri-sm) 0;
  }

  /* ── Vehicles ──────────────────────────────────── */

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

  /* ── Responsive ────────────────────────────────── */

  @media (max-width: 768px) {
    .setting-row {
      flex-direction: column;
      align-items: flex-start;
      gap: var(--space-2);
    }

    .setting-chip {
      min-height: 40px;
      padding: var(--space-2) var(--space-3);
    }
  }
</style>
