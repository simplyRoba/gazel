<script lang="ts">
  let {
    open = false,
    title = "",
    message = "",
    mode = "confirm",
    variant = "warning",
    confirmLabel = "Confirm",
    onconfirm,
    oncancel,
    onclose,
  }: {
    open?: boolean;
    title?: string;
    message?: string;
    mode?: "confirm" | "alert";
    variant?: "danger" | "warning";
    confirmLabel?: string;
    onconfirm?: () => void;
    oncancel?: () => void;
    onclose?: () => void;
  } = $props();

  let dialogEl: HTMLDialogElement | undefined = $state();

  $effect(() => {
    if (!dialogEl) return;
    if (open && !dialogEl.open) {
      dialogEl.showModal();
    } else if (!open && dialogEl.open) {
      dialogEl.close();
    }
  });

  function handleCancel(e: Event) {
    e.preventDefault();
    if (mode === "confirm") {
      oncancel?.();
    } else {
      onclose?.();
    }
  }

  function handleBackdropClick(e: MouseEvent) {
    if (mode === "alert") return;
    if (e.target === dialogEl) {
      oncancel?.();
    }
  }
</script>

<dialog
  bind:this={dialogEl}
  class="modal-dialog"
  oncancel={handleCancel}
  onclick={handleBackdropClick}
>
  <div
    class="modal-content corner-tri"
    style="--_tri-color: {variant === 'danger'
      ? 'var(--color-error)'
      : 'var(--color-accent)'}"
  >
    <h3 class="modal-title">{title}</h3>
    <p class="modal-message">{message}</p>
    <div class="modal-actions">
      {#if mode === "confirm"}
        <button type="button" class="btn btn-secondary" onclick={oncancel}>
          Cancel
        </button>
        <button
          type="button"
          class="btn {variant === 'danger' ? 'btn-danger' : 'btn-primary'}"
          onclick={onconfirm}
        >
          {confirmLabel}
        </button>
      {:else}
        <button
          type="button"
          class="btn {variant === 'danger' ? 'btn-danger' : 'btn-primary'}"
          onclick={onclose}
        >
          OK
        </button>
      {/if}
    </div>
  </div>
</dialog>

<style>
  .modal-dialog {
    border: none;
    padding: 0;
    margin: auto;
    background: transparent;
    max-width: 400px;
    width: calc(100% - var(--space-8));
    position: fixed;
    inset: 0;
    height: fit-content;
  }

  .modal-dialog::backdrop {
    background: color-mix(in srgb, var(--color-bg) 70%, transparent);
  }

  .modal-content {
    background: var(--color-bg-raised);
    border: 1px solid var(--color-border);
    padding: var(--space-6);
    box-shadow: var(--shadow-lg);
  }

  .modal-title {
    font-size: var(--font-lg);
    font-weight: var(--font-weight-semibold);
    margin-bottom: var(--space-3);
  }

  .modal-message {
    font-size: var(--font-sm);
    color: var(--color-text-secondary);
    line-height: var(--line-height-normal);
    margin-bottom: var(--space-6);
  }

  .modal-actions {
    display: flex;
    justify-content: flex-end;
    gap: var(--space-3);
  }
</style>
