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

  import { page } from "$app/state";
  import { resolve } from "$app/paths";
  import { onMount } from "svelte";
  import { LayoutDashboard, Settings, Plus } from "lucide-svelte";
  import { initSettings } from "$lib/stores/settings.svelte";
  import Logo from "$lib/components/Logo.svelte";
  import ToastHost from "$lib/components/ToastHost.svelte";

  let { children } = $props();

  function isActive(href: string): boolean {
    if (href === "/") {
      return (
        page.url.pathname === "/" || page.url.pathname.startsWith("/vehicles")
      );
    }
    return page.url.pathname.startsWith(href);
  }

  function handleCta(): void {
    // TODO: open add fill-up modal (wired in fill-up UI change)
  }

  onMount(() => {
    initSettings();
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
