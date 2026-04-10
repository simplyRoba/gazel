<script lang="ts">
  import type { Fillup, CreateFillup } from "$lib/api";
  import { t } from "$lib/i18n";
  import { getFillupsByVehicle } from "$lib/stores/fillups.svelte";
  import FillupForm from "./FillupForm.svelte";
  import ModalDialog from "./ModalDialog.svelte";

  let {
    open = false,
    vehicleId,
    initial,
    onsave,
    ondelete,
    onclose,
  }: {
    open?: boolean;
    vehicleId: number;
    initial?: Fillup;
    onsave: (data: CreateFillup) => Promise<void>;
    ondelete?: (fillupId: number) => Promise<void>;
    onclose: () => void;
  } = $props();

  let saving = $state(false);
  let saveError = $state<string | null>(null);
  let showDeleteConfirm = $state(false);
  let odoMode = $state<"total" | "trip">("total");

  // Show the odometer mode switcher only in create mode with prior fill-ups
  const showOdoSwitch = $derived(
    !initial && getFillupsByVehicle(vehicleId).length > 0,
  );

  let dialogEl: HTMLDialogElement | undefined = $state();

  $effect(() => {
    if (!dialogEl) return;
    if (open && !dialogEl.open) {
      dialogEl.showModal();
    } else if (!open && dialogEl.open) {
      dialogEl.close();
    }
  });

  // Reset state when modal opens/closes
  $effect(() => {
    if (open) {
      saving = false;
      saveError = null;
      showDeleteConfirm = false;
      odoMode = "total";
    }
  });

  function handleCancel(e: Event) {
    e.preventDefault();
    onclose();
  }

  function handleBackdropClick(e: MouseEvent) {
    if (e.target === dialogEl) {
      onclose();
    }
  }

  async function handleSave(data: CreateFillup) {
    saving = true;
    saveError = null;
    try {
      await onsave(data);
      onclose();
    } catch {
      // onsave threw — store already pushed a toast with details.
      // Show a generic inline message so the user sees the form stayed open.
      saveError = t("fillup.modal.saveError");
    } finally {
      saving = false;
    }
  }

  function handleDeleteRequest() {
    showDeleteConfirm = true;
  }

  async function handleDeleteConfirm() {
    if (!initial || !ondelete) return;
    showDeleteConfirm = false;
    saving = true;
    try {
      await ondelete(initial.id);
      onclose();
    } finally {
      saving = false;
    }
  }
</script>

<dialog
  bind:this={dialogEl}
  class="fillup-modal"
  oncancel={handleCancel}
  onclick={handleBackdropClick}
>
  <div class="modal-body corner-tri">
    <div class="modal-header">
      <h3 class="modal-title">
        {initial ? t("fillup.edit.title") : t("fillup.add.title")}
      </h3>
      {#if showOdoSwitch}
        <div class="segmented">
          <button
            type="button"
            class="segmented-item"
            class:active={odoMode === "total"}
            disabled={saving}
            onclick={() => {
              odoMode = "total";
            }}
          >
            {t("fillup.modal.total")}
          </button>
          <button
            type="button"
            class="segmented-item"
            class:active={odoMode === "trip"}
            disabled={saving}
            onclick={() => {
              odoMode = "trip";
            }}
          >
            {t("fillup.modal.trip")}
          </button>
        </div>
      {/if}
    </div>
    {#if saveError}
      <div class="save-error">{saveError}</div>
    {/if}
    {#key open && initial?.id}
      <FillupForm
        {initial}
        {vehicleId}
        {odoMode}
        onsave={handleSave}
        oncancel={onclose}
        ondelete={initial ? handleDeleteRequest : undefined}
        {saving}
      />
    {/key}
  </div>
</dialog>

<ModalDialog
  open={showDeleteConfirm}
  title={t("fillup.modal.deleteTitle")}
  message={t("fillup.modal.deleteMessage")}
  mode="confirm"
  variant="danger"
  confirmLabel={t("common.delete")}
  onconfirm={handleDeleteConfirm}
  oncancel={() => (showDeleteConfirm = false)}
/>

<style>
  .fillup-modal {
    border: none;
    padding: 0;
    margin: auto;
    background: transparent;
    max-width: 520px;
    width: calc(100% - var(--space-8));
    position: fixed;
    inset: 0;
    height: fit-content;
  }

  .fillup-modal::backdrop {
    background: color-mix(in srgb, var(--color-bg) 70%, transparent);
  }

  .modal-body {
    background: var(--color-bg-raised);
    border: 1px solid var(--color-border);
    padding: var(--space-6);
    box-shadow: var(--shadow-lg);
  }

  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-3);
    margin-bottom: var(--space-5);
  }

  .modal-title {
    font-size: var(--font-lg);
    font-weight: var(--font-weight-semibold);
  }

  .save-error {
    padding: var(--space-3);
    margin-bottom: var(--space-4);
    background: color-mix(in srgb, var(--color-error) 10%, transparent);
    border: 1px solid var(--color-error);
    color: var(--color-error);
    font-size: var(--font-sm);
  }

  @media (max-width: 768px) {
    .fillup-modal {
      max-width: none;
      width: 100%;
      margin: 0;
      margin-top: auto;
      inset: auto 0 0 0;
      border-radius: 0;
    }

    .modal-body {
      padding: var(--space-5);
      padding-bottom: calc(var(--space-5) + env(safe-area-inset-bottom, 0px));
    }
  }
</style>
