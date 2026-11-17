<script lang="ts">
  import {
    ChevronLeft,
    ChevronRight,
    ChevronsLeft,
    ChevronsRight,
  } from "lucide-svelte";
  import { t } from "#lib/i18n/index.js";
  import type { MonthlySelection } from "#lib/monthly-window.js";

  let {
    window,
    onmove,
  }: {
    window: {
      canPrevious: boolean;
      canNext: boolean;
      move: (direction: -1 | 1, step?: 1 | 12) => MonthlySelection | null;
    };
    onmove: (selection: MonthlySelection | null) => void;
  } = $props();
</script>

<div class="segmented segmented-sm navigation">
  <button
    type="button"
    class="segmented-item"
    aria-label={t("charts.previousYear")}
    disabled={!window.canPrevious}
    onclick={() => onmove(window.move(-1, 12))}
  >
    <ChevronsLeft size={14} aria-hidden="true" />
  </button>
  <button
    type="button"
    class="segmented-item"
    aria-label={t("charts.previousPeriod")}
    disabled={!window.canPrevious}
    onclick={() => onmove(window.move(-1))}
  >
    <ChevronLeft size={14} aria-hidden="true" />
  </button>
  <button
    type="button"
    class="segmented-item"
    aria-label={t("charts.nextPeriod")}
    disabled={!window.canNext}
    onclick={() => onmove(window.move(1))}
  >
    <ChevronRight size={14} aria-hidden="true" />
  </button>
  <button
    type="button"
    class="segmented-item"
    aria-label={t("charts.nextYear")}
    disabled={!window.canNext}
    onclick={() => onmove(window.move(1, 12))}
  >
    <ChevronsRight size={14} aria-hidden="true" />
  </button>
</div>

<style>
  .navigation .segmented-item {
    padding-inline: var(--space-2);
  }

  .navigation .segmented-item:disabled {
    opacity: 0.4;
    cursor: default;
    pointer-events: none;
  }

  @media (max-width: 959px) {
    .navigation {
      display: none;
    }
  }
</style>
