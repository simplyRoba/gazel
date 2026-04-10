<script lang="ts">
  import { onDestroy } from "svelte";
  import { SvelteMap } from "svelte/reactivity";
  import { X } from "lucide-svelte";
  import { t } from "$lib/i18n";
  import {
    getVisibleNotifications,
    dismissNotification,
    type Notification,
  } from "$lib/stores/notifications.svelte";

  const AUTO_DISMISS_MS = 3500;

  let timers = new SvelteMap<string, ReturnType<typeof setTimeout>>();

  function startDismiss(n: Notification) {
    if (n.variant === "error") return;
    if (timers.has(n.id)) return;
    timers.set(
      n.id,
      setTimeout(() => {
        dismissNotification(n.id);
        timers.delete(n.id);
      }, AUTO_DISMISS_MS),
    );
  }

  function pauseDismiss(id: string) {
    const timer = timers.get(id);
    if (timer) {
      clearTimeout(timer);
      timers.delete(id);
    }
  }

  function resumeDismiss(n: Notification) {
    if (n.variant === "error") return;
    startDismiss(n);
  }

  // Start timers for new notifications
  $effect(() => {
    const visible = getVisibleNotifications();
    for (const n of visible) {
      startDismiss(n);
    }
  });

  onDestroy(() => {
    for (const timer of timers.values()) {
      clearTimeout(timer);
    }
  });

  function variantClass(variant: string): string {
    return `toast-${variant}`;
  }
</script>

{#if getVisibleNotifications().length > 0}
  <div class="toast-host" aria-label={t("toast.notifications")}>
    {#each getVisibleNotifications() as notification (notification.id)}
      <div
        class="toast corner-tri corner-tri-sm {variantClass(
          notification.variant,
        )}"
        style="--_tri-color: var(--toast-accent)"
        role={notification.variant === "error" ? "alert" : "status"}
        aria-live={notification.variant === "error" ? "assertive" : "polite"}
        onmouseenter={() => pauseDismiss(notification.id)}
        onmouseleave={() => resumeDismiss(notification)}
      >
        <div class="toast-body">
          {#if notification.title}
            <strong class="toast-title">{notification.title}</strong>
          {/if}
          <span class="toast-message">{notification.message}</span>
          {#if notification.action}
            <button
              class="btn btn-ghost toast-action"
              onclick={notification.action.onClick}
            >
              {notification.action.label}
            </button>
          {/if}
        </div>
        <button
          class="toast-close"
          onclick={() => dismissNotification(notification.id)}
          aria-label={t("toast.dismiss")}
        >
          <X size={14} />
        </button>
      </div>
    {/each}
  </div>
{/if}

<style>
  .toast-host {
    position: fixed;
    z-index: 160;
    right: var(--space-4);
    bottom: var(--space-4);
    display: flex;
    flex-direction: column-reverse;
    gap: var(--space-2);
    max-width: 360px;
    pointer-events: none;
  }

  .toast {
    display: flex;
    align-items: flex-start;
    gap: var(--space-3);
    padding: var(--space-3) var(--space-4);
    background: var(--color-bg-raised);
    border: 1px solid var(--color-border);
    border-left: 4px solid var(--toast-accent);
    box-shadow: var(--shadow-md);
    pointer-events: auto;
  }

  .toast-success {
    --toast-accent: var(--color-success);
  }

  .toast-error {
    --toast-accent: var(--color-error);
  }

  .toast-info {
    --toast-accent: var(--color-info);
  }

  .toast-body {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .toast-title {
    font-size: var(--font-sm);
    font-weight: var(--font-weight-semibold);
  }

  .toast-message {
    font-size: var(--font-sm);
    color: var(--color-text-secondary);
  }

  .toast-action {
    font-size: var(--font-xs);
    margin-top: var(--space-1);
    align-self: flex-start;
  }

  .toast-close {
    background: none;
    border: none;
    padding: var(--space-1);
    color: var(--color-text-tertiary);
    cursor: pointer;
    flex-shrink: 0;
  }

  .toast-close:hover {
    color: var(--color-text);
  }

  /* Mobile: top position */
  @media (max-width: 768px) {
    .toast-host {
      top: max(var(--space-3), env(safe-area-inset-top));
      bottom: auto;
      right: var(--space-3);
      left: var(--space-3);
      max-width: none;
      flex-direction: column;
    }
  }
</style>
