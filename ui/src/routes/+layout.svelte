<script lang="ts">
  import "$lib/styles/tokens.css";
  import "$lib/styles/reset.css";
  import "$lib/styles/corner.css";
  import "$lib/styles/buttons.css";
  import "$lib/styles/inputs.css";
  import "$lib/styles/chips.css";
  import "$lib/styles/badges.css";
  import "$lib/styles/cards.css";
  import "$lib/styles/skeletons.css";
  import "$lib/styles/segmented.css";

  import { goto } from "$app/navigation";
  import { page } from "$app/state";
  import { resolve } from "$app/paths";
  import { onMount, untrack } from "svelte";
  import { LayoutDashboard, Settings, Plus, Check } from "lucide-svelte";
  import { initSettings } from "$lib/stores/settings.svelte";
  import { getVehicles, loadVehicles } from "$lib/stores/vehicles.svelte";
  import {
    getActiveVehicleId,
    setActiveVehicle,
    createFillup as storeCreateFillup,
  } from "$lib/stores/fillups.svelte";
  import Logo from "$lib/components/Logo.svelte";
  import ToastHost from "$lib/components/ToastHost.svelte";
  import FillupModal from "$lib/components/FillupModal.svelte";
  import type { CreateFillup, Vehicle } from "$lib/api";
  import { t } from "$lib/i18n";
  import {
    PULL_TO_REFRESH_THRESHOLD,
    isStandalonePwaSession,
    isTouchCapableDevice,
    canStartPullToRefresh,
    hasBlockingPullToRefreshOverlay,
    calculatePullOffset,
    calculateContentOffset,
    getPullIndicatorState,
    shouldTriggerPullToRefresh,
    schedulePullToRefreshReload,
    type PullIndicatorState,
  } from "$lib/pull-to-refresh";

  let { children } = $props();

  // CTA modal state
  let showCtaModal = $state(false);
  let ctaVehicleId = $state<number | null>(null);
  let showVehiclePicker = $state(false);

  function isActive(href: string): boolean {
    if (href === "/") {
      return (
        page.url.pathname === "/" || page.url.pathname.startsWith("/vehicles")
      );
    }
    return page.url.pathname.startsWith(href);
  }

  function handleCta(): void {
    const vehicles = getVehicles();
    if (vehicles.length === 0) {
      goto(resolve("/settings/vehicles/new"));
      return;
    }
    if (vehicles.length === 1) {
      ctaVehicleId = vehicles[0].id;
      // Ensure this vehicle's fill-ups are loaded
      if (getActiveVehicleId() !== vehicles[0].id) {
        setActiveVehicle(vehicles[0].id);
      }
      showCtaModal = true;
      return;
    }
    // Multiple vehicles: show picker
    showVehiclePicker = true;
  }

  function selectVehicleForCta(vehicle: Vehicle): void {
    ctaVehicleId = vehicle.id;
    if (getActiveVehicleId() !== vehicle.id) {
      setActiveVehicle(vehicle.id);
    }
    showVehiclePicker = false;
    showCtaModal = true;
  }

  function closeCtaModal(): void {
    showCtaModal = false;
    ctaVehicleId = null;
  }

  function closeVehiclePicker(): void {
    showVehiclePicker = false;
  }

  async function handleCtaSave(data: CreateFillup): Promise<void> {
    if (!ctaVehicleId) return;
    const result = await storeCreateFillup(ctaVehicleId, data);
    if (!result) {
      throw new Error("Save failed");
    }
  }

  let pickerDialogEl: HTMLDialogElement | undefined = $state();

  $effect(() => {
    if (!pickerDialogEl) return;
    if (showVehiclePicker && !pickerDialogEl.open) {
      pickerDialogEl.showModal();
    } else if (!showVehiclePicker && pickerDialogEl.open) {
      pickerDialogEl.close();
    }
  });

  // ── Pull-to-refresh state ──────────────────────────────
  let canUsePullToRefresh = $state(false);
  let gestureActive = $state(false);
  let touchStartY: number | null = $state(null);
  let rawPullDistance = $state(0);
  let pullOffset = $state(0);
  let contentOffset = $state(0);
  let pullIndicatorState = $state<PullIndicatorState>("idle");
  let reloadTimeoutId: number | null = $state(null);

  let pullIndicatorVisible = $derived(pullIndicatorState !== "idle");
  let pullLabel = $derived(
    pullIndicatorState === "refreshing"
      ? t("pullToRefresh.refreshing")
      : pullIndicatorState === "release"
        ? t("pullToRefresh.release")
        : t("pullToRefresh.pulling"),
  );
  let spinnerRotation = $derived(
    Math.min(rawPullDistance / PULL_TO_REFRESH_THRESHOLD, 1) * 360,
  );

  function resetPullGesture(): void {
    gestureActive = false;
    touchStartY = null;
    rawPullDistance = 0;
    pullOffset = 0;
    contentOffset = 0;
    pullIndicatorState = "idle";
  }

  function handleTouchStart(e: TouchEvent): void {
    if (e.touches.length !== 1) return;
    if (
      !canStartPullToRefresh({
        standalone: canUsePullToRefresh,
        touchCapable: true,
        pathname: page.url.pathname,
        scrollTop:
          document.documentElement.scrollTop || document.body.scrollTop,
        overlayOpen: hasBlockingPullToRefreshOverlay(document),
      })
    )
      return;

    if (reloadTimeoutId !== null) {
      clearTimeout(reloadTimeoutId);
      reloadTimeoutId = null;
    }

    touchStartY = e.touches[0].clientY;
    gestureActive = true;
    rawPullDistance = 0;
    pullOffset = 0;
    contentOffset = 0;
    pullIndicatorState = "idle";
  }

  function handleTouchMove(e: TouchEvent): void {
    if (!gestureActive || touchStartY === null) return;
    if (e.touches.length !== 1 || hasBlockingPullToRefreshOverlay(document)) {
      resetPullGesture();
      return;
    }

    const distance = e.touches[0].clientY - touchStartY;

    if (distance <= 0) {
      rawPullDistance = 0;
      pullOffset = 0;
      contentOffset = 0;
      pullIndicatorState = "idle";
      return;
    }

    rawPullDistance = distance;
    pullOffset = calculatePullOffset(distance);
    contentOffset = calculateContentOffset(distance);
    pullIndicatorState = getPullIndicatorState(distance);
    e.preventDefault();
  }

  function handleTouchEnd(): void {
    if (!gestureActive) return;

    if (shouldTriggerPullToRefresh(rawPullDistance)) {
      gestureActive = false;
      touchStartY = null;
      pullIndicatorState = "refreshing";
      reloadTimeoutId = schedulePullToRefreshReload(window, () =>
        window.location.reload(),
      );
    } else {
      resetPullGesture();
    }
  }

  function handleTouchCancel(): void {
    resetPullGesture();
  }

  // Reset gesture on route change (unless refreshing)
  $effect(() => {
    const _pathname = page.url.pathname;
    if (untrack(() => pullIndicatorState) !== "refreshing") {
      resetPullGesture();
    }
  });

  // Reset gesture if capability lost
  $effect(() => {
    if (
      !canUsePullToRefresh &&
      untrack(() => pullIndicatorState) !== "refreshing"
    ) {
      resetPullGesture();
    }
  });

  onMount(() => {
    initSettings().then(() => loadVehicles());

    // Detect pull-to-refresh capability
    const win = window as unknown as {
      matchMedia: (q: string) => { matches: boolean };
      navigator: { standalone?: boolean; maxTouchPoints: number };
      ontouchstart?: unknown;
    };
    canUsePullToRefresh =
      isStandalonePwaSession(win) && isTouchCapableDevice(win);

    if (canUsePullToRefresh) {
      window.addEventListener("touchstart", handleTouchStart, {
        passive: true,
      });
      window.addEventListener("touchmove", handleTouchMove, {
        passive: false,
      });
      window.addEventListener("touchend", handleTouchEnd);
      window.addEventListener("touchcancel", handleTouchCancel);
    }

    return () => {
      window.removeEventListener("touchstart", handleTouchStart);
      window.removeEventListener("touchmove", handleTouchMove);
      window.removeEventListener("touchend", handleTouchEnd);
      window.removeEventListener("touchcancel", handleTouchCancel);
    };
  });
</script>

<div class="app-shell">
  <div class="app">
    <!-- Sidebar (tablet+) -->
    <nav class="sidebar">
      <div class="sidebar-logo">
        <Logo size={36} />
      </div>

      <button
        class="sidebar-cta corner-tri-hover corner-tri-sm"
        style="--_tri-color: var(--color-text-inverse)"
        onclick={handleCta}
      >
        <Plus size={20} />
        <span class="cta-label">{t("nav.fillup")}</span>
      </button>

      <a
        href={resolve("/")}
        class="nav-item corner-tri-hover corner-tri-sm"
        class:active={isActive("/")}
      >
        <LayoutDashboard size={20} />
        <span class="nav-label">{t("nav.dashboard")}</span>
      </a>

      <div class="spacer"></div>

      <a
        href={resolve("/settings")}
        class="nav-item corner-tri-hover corner-tri-sm"
        class:active={isActive("/settings")}
      >
        <Settings size={20} />
        <span class="nav-label">{t("nav.settings")}</span>
      </a>
    </nav>

    <!-- Bottom bar (mobile) -->
    <nav class="bottom-bar">
      <a href={resolve("/")} class="nav-item" class:active={isActive("/")}>
        <LayoutDashboard size={20} />
        <span class="nav-label">{t("nav.dashboard")}</span>
      </a>

      <button class="bottom-cta" onclick={handleCta}>
        <Plus size={22} />
      </button>

      <a
        href={resolve("/settings")}
        class="nav-item"
        class:active={isActive("/settings")}
      >
        <Settings size={20} />
        <span class="nav-label">{t("nav.settings")}</span>
      </a>
    </nav>

    <!-- Pull-to-refresh indicator -->
    <div
      class="pull-indicator"
      class:visible={pullIndicatorVisible}
      class:armed={pullIndicatorState === "release"}
      class:refreshing={pullIndicatorState === "refreshing"}
      class:settling={!gestureActive && pullIndicatorState !== "refreshing"}
      style:transform="translateY({pullIndicatorVisible
        ? Math.min(pullOffset, PULL_TO_REFRESH_THRESHOLD) - 68
        : -100}px)"
    >
      <span class="pull-indicator-label">{pullLabel}</span>
      {#if pullIndicatorState === "release"}
        <span class="pull-indicator-check"><Check size={20} /></span>
      {:else}
        <span
          class="pull-indicator-spinner"
          class:spinning={pullIndicatorState === "refreshing"}
          style:transform={pullIndicatorState === "pulling"
            ? `rotate(${spinnerRotation}deg)`
            : undefined}
        ></span>
      {/if}
    </div>

    <!-- Content -->
    <main
      class="content"
      class:settling={!gestureActive && pullIndicatorState !== "refreshing"}
      style:margin-top={contentOffset > 0 ? `${contentOffset}px` : undefined}
    >
      {@render children()}
    </main>

    <ToastHost />
  </div>
</div>

<!-- Vehicle picker dialog (CTA with multiple vehicles) -->
<dialog
  bind:this={pickerDialogEl}
  class="picker-dialog"
  oncancel={(e) => {
    e.preventDefault();
    closeVehiclePicker();
  }}
  onclick={(e) => {
    if (e.target === pickerDialogEl) closeVehiclePicker();
  }}
>
  <div class="picker-body corner-tri">
    <h3 class="picker-title">{t("nav.selectVehicle")}</h3>
    <div class="picker-list">
      {#each getVehicles() as vehicle (vehicle.id)}
        <button
          class="picker-item"
          onclick={() => selectVehicleForCta(vehicle)}
        >
          {vehicle.name}
          {#if vehicle.make || vehicle.model}
            <span class="picker-meta">
              {[vehicle.make, vehicle.model].filter(Boolean).join(" ")}
            </span>
          {/if}
        </button>
      {/each}
    </div>
    <div class="picker-actions">
      <button class="btn btn-secondary" onclick={closeVehiclePicker}
        >{t("common.cancel")}</button
      >
    </div>
  </div>
</dialog>

<!-- CTA fill-up modal -->
{#if ctaVehicleId}
  <FillupModal
    open={showCtaModal}
    vehicleId={ctaVehicleId}
    onsave={handleCtaSave}
    onclose={closeCtaModal}
  />
{/if}

<style>
  /* ── Layout tokens ──────────────────────────── */
  :global(:root) {
    --nav-bottom-height: 56px;
    --safe-area-bottom: env(safe-area-inset-bottom, 0px);
    --nav-bottom-total: calc(
      var(--nav-bottom-height) + var(--safe-area-bottom)
    );
    --sidebar-width: 64px;
    --content-width-narrow: 640px;
    --content-width-default: 800px;
    --content-width-wide: 1200px;
  }

  /* ── App shell ──────────────────────────────── */
  .app {
    min-height: 100dvh;
  }

  /* ── Sidebar (hidden on mobile, shown tablet+) */
  .sidebar {
    display: none;
    position: fixed;
    top: 0;
    left: 0;
    bottom: 0;
    width: var(--sidebar-width);
    background: var(--color-nav-bg);
    border-right: 1px solid var(--color-nav-border);
    flex-direction: column;
    align-items: center;
    padding: var(--space-4) 0;
    z-index: 100;
  }

  .sidebar-logo {
    margin-bottom: var(--space-4);
    color: var(--color-brand-gold-1);
  }

  .sidebar-cta {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--space-2);
    margin: 0 var(--space-2) var(--space-4);
    padding: var(--space-2);
    width: calc(var(--sidebar-width) - var(--space-4));
    height: 44px;
    background: var(--color-accent);
    color: var(--color-text-inverse);
    border: none;
    font-size: var(--font-sm);
    font-weight: var(--font-weight-medium);
    box-shadow: var(--shadow-sm);
    transition: background var(--transition-fast);
  }

  .sidebar-cta:hover {
    background: var(--color-accent-hover);
  }

  .cta-label {
    display: none;
  }

  .nav-item {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 2px;
    padding: var(--space-2);
    color: var(--color-nav-text);
    text-decoration: none;
    font-size: 11px;
    width: 48px;
    height: 48px;
    transition:
      background var(--transition-fast),
      color var(--transition-fast);
  }

  .nav-item:hover {
    background: var(--color-nav-hover);
    color: var(--color-brand-gold-2);
  }

  .nav-item.active {
    background: rgba(212, 165, 106, 0.12);
    color: var(--color-nav-text-active);
  }

  .sidebar .nav-label {
    display: none;
  }

  .spacer {
    flex: 1;
  }

  /* ── Bottom bar (mobile) ────────────────────── */
  .bottom-bar {
    position: fixed;
    bottom: 0;
    left: 0;
    right: 0;
    height: var(--nav-bottom-total);
    background: var(--color-nav-bg);
    border-top: 1px solid var(--color-nav-border);
    display: flex;
    justify-content: space-around;
    align-items: flex-start;
    padding-top: var(--space-2);
    z-index: 100;
  }

  .bottom-bar .nav-item {
    width: auto;
    height: auto;
    padding: var(--space-1) var(--space-3);
    font-size: 10px;
  }

  .bottom-bar .nav-label {
    display: block;
  }

  .bottom-cta {
    width: 52px;
    height: 52px;
    background: var(--color-accent);
    color: var(--color-text-inverse);
    border: none;
    display: flex;
    align-items: center;
    justify-content: center;
    margin-top: -22px;
    box-shadow: var(--shadow-md);
    transform: rotate(45deg);
    transition: background var(--transition-fast);
  }

  .bottom-cta :global(svg) {
    transform: rotate(-45deg);
  }

  /* ── Content ────────────────────────────────── */
  .content {
    padding: var(--space-4);
    padding-bottom: calc(var(--nav-bottom-total) + var(--space-4));
  }

  /* ── Tablet (>768px) ────────────────────────── */
  @media (min-width: 769px) {
    .sidebar {
      display: flex;
    }

    .bottom-bar {
      display: none;
    }

    .content {
      margin-left: var(--sidebar-width);
      padding: var(--space-6);
    }
  }

  /* ── Widescreen (>=1280px) ──────────────────── */
  @media (min-width: 1280px) {
    :global(:root) {
      --sidebar-width: 200px;
      --content-width-narrow: 720px;
      --content-width-default: 960px;
      --content-width-wide: 1400px;
    }

    .sidebar-logo {
      margin-bottom: var(--space-6);
    }

    .sidebar-logo :global(svg) {
      width: 80px;
      height: 80px;
    }

    .sidebar-cta {
      width: calc(var(--sidebar-width) - var(--space-4) * 2);
      padding: var(--space-2) var(--space-3);
    }

    .cta-label {
      display: inline;
    }

    .nav-item {
      flex-direction: row;
      width: calc(var(--sidebar-width) - var(--space-4) * 2);
      height: auto;
      padding: var(--space-2) var(--space-3);
      gap: var(--space-2);
      justify-content: flex-start;
    }

    .sidebar .nav-label {
      display: block;
      font-size: var(--font-sm);
    }

    .content {
      padding: var(--space-8);
    }
  }

  /* ── Vehicle picker dialog ────────────────────── */
  :global(.picker-dialog) {
    border: none;
    padding: 0;
    margin: auto;
    background: transparent;
    max-width: 360px;
    width: calc(100% - var(--space-8));
    position: fixed;
    inset: 0;
    height: fit-content;
  }

  :global(.picker-dialog::backdrop) {
    background: color-mix(in srgb, var(--color-bg) 70%, transparent);
  }

  :global(.picker-body) {
    background: var(--color-bg-raised);
    border: 1px solid var(--color-border);
    padding: var(--space-6);
    box-shadow: var(--shadow-lg);
  }

  :global(.picker-title) {
    font-size: var(--font-lg);
    font-weight: var(--font-weight-semibold);
    margin-bottom: var(--space-4);
  }

  :global(.picker-list) {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    margin-bottom: var(--space-5);
  }

  :global(.picker-item) {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-3);
    background: var(--color-bg);
    border: 1px solid var(--color-border);
    font-size: var(--font-sm);
    font-weight: var(--font-weight-medium);
    cursor: pointer;
    transition: border-color var(--transition-fast);
    text-align: left;
  }

  :global(.picker-item:hover) {
    border-color: var(--color-accent);
  }

  :global(.picker-meta) {
    font-weight: var(--font-weight-normal);
    color: var(--color-text-secondary);
    font-size: var(--font-xs);
  }

  :global(.picker-actions) {
    display: flex;
    justify-content: flex-end;
  }

  /* ── Mobile (<=768px) ───────────────────────── */
  @media (max-width: 768px) {
    .sidebar {
      display: none;
    }

    .bottom-bar .nav-item:hover,
    .bottom-bar .nav-item.active {
      background: none;
    }

    .bottom-bar .nav-item.active {
      color: var(--color-nav-text-active);
    }
  }

  /* ── Pull-to-refresh indicator ─────────────── */
  .pull-indicator {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    z-index: 160;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 6px;
    margin: 0 auto;
    padding: 12px;
    color: var(--color-text-secondary);
    font-size: 15px;
    font-weight: var(--font-weight-medium);
    pointer-events: none;
  }

  .pull-indicator.settling {
    transition: transform 0.18s ease;
  }

  .pull-indicator-label {
    user-select: none;
  }

  .pull-indicator-spinner {
    display: block;
    width: 22px;
    height: 22px;
    border: 2.5px solid var(--color-border);
    border-top-color: var(--color-accent);
    border-radius: 999px;
  }

  .pull-indicator-spinner.spinning {
    animation: pull-refresh-spin 0.8s linear infinite;
  }

  .pull-indicator-check {
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--color-accent);
  }

  .content.settling {
    transition: margin-top 0.18s ease;
  }

  @keyframes pull-refresh-spin {
    from {
      transform: rotate(0deg);
    }
    to {
      transform: rotate(360deg);
    }
  }
</style>
