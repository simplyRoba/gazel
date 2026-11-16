<script lang="ts">
  import { ChevronLeft, ChevronRight } from "lucide-svelte";
  import { t } from "#lib/i18n/index.js";
  import type { MonthlySelection } from "#lib/monthly-window.js";

  let {
    window,
    locale,
    onmove,
  }: {
    window: {
      points: { period: string }[];
      canPrevious: boolean;
      canNext: boolean;
      move: (direction: -1 | 1) => MonthlySelection | null;
    };
    locale: string;
    onmove: (selection: MonthlySelection | null) => void;
  } = $props();

  function format(period: string) {
    const [year, month] = period.split("-").map(Number);
    return new Date(year, month - 1, 1).toLocaleDateString(locale, {
      month: "short",
      year: "numeric",
    });
  }
</script>

<div class="navigation">
  <button
    type="button"
    class="btn btn-icon"
    aria-label={t("charts.previousPeriod")}
    disabled={!window.canPrevious}
    onclick={() => onmove(window.move(-1))}
  >
    <ChevronLeft size={16} aria-hidden="true" />
  </button>
  <span
    >{format(window.points[0].period)} – {format(
      window.points[window.points.length - 1].period,
    )}</span
  >
  <button
    type="button"
    class="btn btn-icon"
    aria-label={t("charts.nextPeriod")}
    disabled={!window.canNext}
    onclick={() => onmove(window.move(1))}
  >
    <ChevronRight size={16} aria-hidden="true" />
  </button>
</div>

<style>
  .navigation {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.5rem;
    font-size: var(--font-xs);
    white-space: nowrap;
  }
</style>
