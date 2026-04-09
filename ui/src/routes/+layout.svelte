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
  import { onMount } from "svelte";
  import { LayoutDashboard, Settings, Plus } from "lucide-svelte";
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

  onMount(async () => {
    await initSettings();
    await loadVehicles();
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
        class="sidebar-cta corner-tri corner-tri-sm"
        style="--_tri-color: var(--color-text-inverse)"
        onclick={handleCta}
      >
        <Plus size={20} />
        <span class="cta-label">Fill-up</span>
      </button>

      <a
        href={resolve("/")}
        class="nav-item corner-tri-hover corner-tri-sm"
        class:active={isActive("/")}
      >
        <LayoutDashboard size={20} />
        <span class="nav-label">Dashboard</span>
      </a>

      <div class="spacer"></div>

      <a
        href={resolve("/settings")}
        class="nav-item corner-tri-hover corner-tri-sm"
        class:active={isActive("/settings")}
      >
        <Settings size={20} />
        <span class="nav-label">Settings</span>
      </a>
    </nav>

    <!-- Bottom bar (mobile) -->
    <nav class="bottom-bar">
      <a href={resolve("/")} class="nav-item" class:active={isActive("/")}>
        <LayoutDashboard size={20} />
        <span class="nav-label">Dashboard</span>
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
        <span class="nav-label">Settings</span>
      </a>
    </nav>

    <!-- Content -->
    <main class="content">
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
    <h3 class="picker-title">Select vehicle</h3>
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
        >Cancel</button
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
    background: var(--color-bg-raised);
    border-right: 1px solid var(--color-border);
    flex-direction: column;
    align-items: center;
    padding: var(--space-4) 0;
    z-index: 100;
  }

  .sidebar-logo {
    margin-bottom: var(--space-4);
    color: var(--color-accent-text);
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
    color: var(--color-text-secondary);
    text-decoration: none;
    font-size: 11px;
    width: 48px;
    height: 48px;
    transition: background var(--transition-fast);
  }

  .nav-item:hover {
    background: var(--color-bg-sunken);
  }

  .nav-item.active {
    background: var(--color-accent-subtle);
    color: var(--color-accent-text);
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
    background: var(--color-bg-raised);
    border-top: 1px solid var(--color-border);
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
    width: 42px;
    height: 42px;
    background: var(--color-accent);
    color: var(--color-text-inverse);
    border: none;
    display: flex;
    align-items: center;
    justify-content: center;
    margin-top: -16px;
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
      color: var(--color-accent-text);
    }
  }
</style>
