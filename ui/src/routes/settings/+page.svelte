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
    Download,
    Upload,
    Database,
  } from "lucide-svelte";
  import PageContainer from "$lib/components/PageContainer.svelte";
  import EmptyState from "$lib/components/EmptyState.svelte";
  import ModalDialog from "$lib/components/ModalDialog.svelte";
  import type { Vehicle, ImportMode, ImportPreviewResult } from "$lib/api";
  import {
    exportAll,
    exportVehicle,
    previewImport,
    importData,
    ApiError,
  } from "$lib/api";
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
  import { pushNotification } from "$lib/stores/notifications.svelte";
  import { clearCache as clearFillupCache } from "$lib/stores/fillups.svelte";
  import { clearCache as clearStatsCache } from "$lib/stores/stats.svelte";
  import { t } from "$lib/i18n";

  let deleteTarget = $state<Vehicle | null>(null);

  // ── Export state ──────────────────────────────────
  let exporting = $state(false);

  // ── Import state ──────────────────────────────────
  let importMode = $state<ImportMode>("replace");
  let importFileData = $state<unknown>(null);
  let importFileName = $state<string>("");
  let importPreview = $state<ImportPreviewResult | null>(null);
  let importError = $state<string | null>(null);
  let importing = $state(false);
  let previewing = $state(false);

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

  // ── Export handlers ────────────────────────────────

  async function handleExportAll() {
    exporting = true;
    try {
      await exportAll();
      pushNotification({
        variant: "success",
        message: t("settings.export.success"),
      });
    } catch (e) {
      const msg =
        e instanceof ApiError ? e.message : t("settings.export.failed");
      pushNotification({ variant: "error", message: msg });
    } finally {
      exporting = false;
    }
  }

  async function handleExportVehicle(id: number) {
    try {
      await exportVehicle(id);
    } catch (e) {
      const msg =
        e instanceof ApiError ? e.message : t("settings.export.failed");
      pushNotification({ variant: "error", message: msg });
    }
  }

  // ── Import handlers ────────────────────────────────

  async function handleFileSelect(event: Event) {
    importError = null;
    importPreview = null;
    const input = event.target as HTMLInputElement;
    const file = input.files?.[0];
    if (!file) return;
    importFileName = file.name;

    try {
      const text = await file.text();
      const parsed = JSON.parse(text);
      importFileData = parsed;

      // Auto-preview
      previewing = true;
      const result = await previewImport(parsed, importMode);
      importPreview = result;
    } catch (e) {
      if (e instanceof ApiError) {
        importError = e.message;
      } else if (e instanceof SyntaxError) {
        importError = t("settings.import.invalidJson");
      } else {
        importError = t("settings.import.readFailed");
      }
      importFileData = null;
    } finally {
      previewing = false;
      // Reset file input so the same file can be re-selected
      input.value = "";
    }
  }

  async function handleModeChange(mode: ImportMode) {
    importMode = mode;
    // Re-preview with new mode if we have data
    if (importFileData) {
      importError = null;
      previewing = true;
      try {
        importPreview = await previewImport(importFileData, mode);
      } catch (e) {
        importError =
          e instanceof ApiError
            ? e.message
            : t("settings.import.previewFailed");
        importPreview = null;
      } finally {
        previewing = false;
      }
    }
  }

  async function handleImportConfirm() {
    if (!importFileData) return;
    importing = true;
    importError = null;
    try {
      const result = await importData(importFileData, importMode);
      const parts: string[] = [];
      if ("vehicles_created" in result) {
        parts.push(
          t("settings.import.vehiclesCreated", {
            count: result.vehicles_created,
          }),
        );
      }
      if ("vehicles_updated" in result && result.vehicles_updated > 0) {
        parts.push(
          t("settings.import.vehiclesUpdated", {
            count: result.vehicles_updated,
          }),
        );
      }
      if ("fillups_created" in result) {
        parts.push(
          t("settings.import.fillupsCreated", {
            count: result.fillups_created,
          }),
        );
      }
      if ("fillups_skipped" in result && result.fillups_skipped > 0) {
        parts.push(
          t("settings.import.fillupsSkipped", {
            count: result.fillups_skipped,
          }),
        );
      }
      pushNotification({
        variant: "success",
        message: t("settings.import.success", { summary: parts.join(", ") }),
      });
      // Reset import state
      importFileData = null;
      importPreview = null;
      importFileName = "";
      // Clear stale caches and refresh vehicles
      clearFillupCache();
      clearStatsCache();
      loadVehicles();
    } catch (e) {
      importError =
        e instanceof ApiError ? e.message : t("settings.import.importFailed");
    } finally {
      importing = false;
    }
  }

  function handleImportCancel() {
    importFileData = null;
    importPreview = null;
    importFileName = "";
    importError = null;
  }
</script>

<PageContainer>
  <h1 class="page-title">{t("settings.title")}</h1>
  <p class="page-subtitle">{t("settings.subtitle")}</p>

  <div class="settings-grid">
    <!-- Display -->
    <section class="section corner-tri corner-tri-sm">
      <h2 class="section-title">
        <Sun size={14} />
        {t("settings.display")}
      </h2>

      <div class="setting-row">
        <span class="setting-label">{t("settings.theme")}</span>
        <div class="segmented">
          <button
            class="segmented-item"
            class:active={getThemePreference() === "light"}
            onclick={() => handleTheme("light")}
          >
            <Sun size={14} />
            {t("settings.theme.light")}
          </button>
          <button
            class="segmented-item"
            class:active={getThemePreference() === "dark"}
            onclick={() => handleTheme("dark")}
          >
            <Moon size={14} />
            {t("settings.theme.dark")}
          </button>
          <button
            class="segmented-item"
            class:active={getThemePreference() === "system"}
            onclick={() => handleTheme("system")}
          >
            <Monitor size={14} />
            {t("settings.theme.system")}
          </button>
        </div>
      </div>

      <div class="setting-row">
        <span class="setting-label">{t("settings.language")}</span>
        <div class="segmented">
          <button
            class="segmented-item"
            class:active={getSettings().locale === "en"}
            onclick={() => handleLocale("en")}
          >
            {t("settings.language.en")}
          </button>
          <button
            class="segmented-item"
            class:active={getSettings().locale === "de"}
            onclick={() => handleLocale("de")}
          >
            {t("settings.language.de")}
          </button>
        </div>
      </div>
    </section>

    <!-- Units -->
    <section class="section corner-tri corner-tri-sm">
      <h2 class="section-title">
        <Ruler size={14} />
        {t("settings.units")}
      </h2>

      <div class="setting-row">
        <span class="setting-label">{t("settings.units.system")}</span>
        <div class="segmented">
          <button
            class="segmented-item"
            class:active={getSettings().unit_system === "metric"}
            onclick={() => handleUnitSystem("metric")}
          >
            {t("settings.units.metric")}
          </button>
          <button
            class="segmented-item"
            class:active={getSettings().unit_system === "imperial"}
            onclick={() => handleUnitSystem("imperial")}
          >
            {t("settings.units.imperial")}
          </button>
          <button
            class="segmented-item"
            class:active={getSettings().unit_system === "custom"}
            onclick={() => handleUnitSystem("custom")}
          >
            {t("settings.units.custom")}
          </button>
        </div>
      </div>

      {#if getSettings().unit_system === "custom"}
        <div class="setting-row">
          <span class="setting-label">{t("settings.units.distance")}</span>
          <div class="segmented">
            <button
              class="segmented-item"
              class:active={getSettings().distance_unit === "km"}
              onclick={() => handleDistanceUnit("km")}
            >
              {t("settings.units.kilometers")}
            </button>
            <button
              class="segmented-item"
              class:active={getSettings().distance_unit === "mi"}
              onclick={() => handleDistanceUnit("mi")}
            >
              {t("settings.units.miles")}
            </button>
          </div>
        </div>

        <div class="setting-row">
          <span class="setting-label">{t("settings.units.volume")}</span>
          <div class="segmented">
            <button
              class="segmented-item"
              class:active={getSettings().volume_unit === "l"}
              onclick={() => handleVolumeUnit("l")}
            >
              {t("settings.units.liters")}
            </button>
            <button
              class="segmented-item"
              class:active={getSettings().volume_unit === "gal"}
              onclick={() => handleVolumeUnit("gal")}
            >
              {t("settings.units.gallons")}
            </button>
          </div>
        </div>
      {/if}

      <div class="setting-row">
        <span class="setting-label">{t("settings.currency")}</span>
        <div class="segmented">
          {#each currencies as c (c.code)}
            <button
              class="segmented-item"
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
        {t("settings.vehicles")}
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
          heading={t("settings.vehicles.empty.title")}
          description={t("settings.vehicles.empty.description")}
        >
          {#snippet action()}
            <a href={resolve("/settings/vehicles/new")} class="btn btn-primary">
              <Plus size={16} />
              {t("settings.vehicles.add")}
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
                <button
                  class="btn btn-icon"
                  title={t("settings.vehicles.export")}
                  onclick={() => handleExportVehicle(vehicle.id)}
                >
                  <Download size={16} />
                </button>
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
            {t("settings.vehicles.add")}
          </a>
        </div>
      {/if}
    </section>

    <!-- Data -->
    <section class="section corner-tri corner-tri-sm">
      <h2 class="section-title">
        <Database size={14} />
        {t("settings.data")}
      </h2>

      <div class="setting-row">
        <div class="data-row-info">
          <span class="setting-label">{t("settings.export")}</span>
          <span class="data-description"
            >{t("settings.export.description")}</span
          >
        </div>
        <button
          class="btn btn-secondary"
          disabled={exporting}
          onclick={handleExportAll}
        >
          <Download size={14} />
          {exporting
            ? t("settings.export.exporting")
            : t("settings.export.button")}
        </button>
      </div>

      <div class="setting-row data-import-row">
        <div class="data-row-info">
          <span class="setting-label">{t("settings.import")}</span>
          <span class="data-description"
            >{t("settings.import.description")}</span
          >
        </div>

        {#if !importPreview && !importError}
          <label class="btn btn-secondary import-file-btn">
            <Upload size={14} />
            {previewing
              ? t("settings.import.reading")
              : t("settings.import.chooseFile")}
            <input
              type="file"
              accept=".json,application/json"
              onchange={handleFileSelect}
              hidden
            />
          </label>
        {/if}
      </div>

      {#if importError}
        <div class="import-error">
          <p>{importError}</p>
          <button class="btn btn-secondary btn-sm" onclick={handleImportCancel}>
            {t("common.dismiss")}
          </button>
        </div>
      {/if}

      {#if importPreview}
        <div class="import-preview card">
          <h3 class="import-preview-title">{t("settings.import.preview")}</h3>
          <p class="import-preview-file">{importFileName}</p>

          <div class="import-mode-row">
            <span class="setting-label">{t("settings.import.mode")}</span>
            <div class="segmented">
              <button
                class="segmented-item"
                class:active={importMode === "replace"}
                onclick={() => handleModeChange("replace")}
              >
                {t("settings.import.replace")}
              </button>
              <button
                class="segmented-item"
                class:active={importMode === "merge"}
                onclick={() => handleModeChange("merge")}
              >
                {t("settings.import.merge")}
              </button>
            </div>
          </div>

          {#if importMode === "replace"}
            <div class="import-summary">
              <p>
                {#if "vehicles" in importPreview}
                  {t("settings.import.vehiclesCount", {
                    count: importPreview.vehicles,
                  })}, {t("settings.import.fillupsCount", {
                    count: importPreview.fillups,
                  })}
                  {t("settings.import.willBeImported")}
                {/if}
              </p>
              <p class="import-warning">
                {t("settings.import.replaceWarning")}
              </p>
            </div>
          {:else}
            <div class="import-summary">
              {#if "vehicles_new" in importPreview}
                <p>
                  {t("settings.import.newVehicles", {
                    count: importPreview.vehicles_new,
                  })}, {t("settings.import.existingVehicles", {
                    count: importPreview.vehicles_existing,
                  })}
                </p>
                <p>
                  {t("settings.import.newFillups", {
                    count: importPreview.fillups_new,
                  })}, {t("settings.import.duplicateFillups", {
                    count: importPreview.fillups_existing,
                  })}
                </p>
              {/if}
            </div>
          {/if}

          <div class="import-actions">
            <button class="btn btn-secondary" onclick={handleImportCancel}>
              {t("common.cancel")}
            </button>
            <button
              class="btn btn-primary"
              disabled={importing}
              onclick={handleImportConfirm}
            >
              {importing
                ? t("settings.import.importing")
                : t("settings.import.confirm")}
            </button>
          </div>
        </div>
      {/if}
    </section>
  </div>
</PageContainer>

<ModalDialog
  open={!!deleteTarget}
  title={t("settings.deleteVehicle.title")}
  message={deleteTarget
    ? t("settings.deleteVehicle.message", { name: deleteTarget.name })
    : ""}
  mode="confirm"
  variant="danger"
  confirmLabel={t("common.delete")}
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

  /* ── Data section ───────────────────────────────── */

  .data-row-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .data-description {
    font-size: var(--font-xs);
    color: var(--color-text-tertiary);
  }

  .data-import-row {
    flex-wrap: wrap;
  }

  .import-file-btn {
    cursor: pointer;
  }

  .import-error {
    background: var(--color-surface-danger, rgba(220, 38, 38, 0.08));
    border: 1px solid var(--color-border-danger, rgba(220, 38, 38, 0.2));
    border-radius: var(--radius-md);
    padding: var(--space-3) var(--space-4);
    margin-top: var(--space-2);
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: var(--space-3);
  }

  .import-error p {
    font-size: var(--font-sm);
    color: var(--color-text-danger, #dc2626);
    margin: 0;
  }

  .import-preview {
    margin-top: var(--space-3);
    padding: var(--space-4);
  }

  .import-preview-title {
    font-size: var(--font-md);
    font-weight: var(--font-weight-semibold);
    margin-bottom: var(--space-1);
  }

  .import-preview-file {
    font-size: var(--font-xs);
    color: var(--color-text-tertiary);
    margin-bottom: var(--space-3);
  }

  .import-mode-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--space-2) 0;
    margin-bottom: var(--space-2);
  }

  .import-summary {
    font-size: var(--font-sm);
    color: var(--color-text-secondary);
    margin-bottom: var(--space-3);
  }

  .import-summary p {
    margin: var(--space-1) 0;
  }

  .import-warning {
    color: var(--color-text-danger, #dc2626);
    font-weight: var(--font-weight-medium);
  }

  .import-actions {
    display: flex;
    justify-content: flex-end;
    gap: var(--space-2);
  }

  /* ── Responsive ────────────────────────────────── */

  @media (max-width: 768px) {
    .setting-row {
      flex-direction: column;
      align-items: flex-start;
      gap: var(--space-2);
    }

    .import-mode-row {
      flex-direction: column;
      align-items: flex-start;
      gap: var(--space-2);
    }
  }
</style>
