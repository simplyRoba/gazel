<script lang="ts">
  import { scaleBand } from "d3-scale";
  import type { PeriodModes } from "#lib/api.js";
  import { t } from "#lib/i18n/index.js";
  import { toPeriodData } from "#lib/charts.js";
  import { monthlyWindow, type MonthlySelection } from "#lib/monthly-window.js";
  import PeriodNavigation from "./PeriodNavigation.svelte";
  import { formatCurrency } from "#lib/format.js";
  import { getSettings } from "#lib/stores/settings.svelte.js";
  import ChartCard from "./ChartCard.svelte";
  import Bar from "./Bar.svelte";
  import AxisX from "./AxisX.svelte";
  import AxisY from "./AxisY.svelte";
  import Tooltip from "./Tooltip.svelte";

  let {
    series,
    currency,
    vehicleId = null,
    selection = $bindable(null),
  }: {
    series: PeriodModes | null;
    currency: string;
    vehicleId?: number | null;
    selection?: MonthlySelection | null;
  } = $props();

  const settings = $derived(getSettings());

  type CostMode = "monthly" | "yearly";
  let mode: CostMode = $state("monthly");

  const window = $derived(
    monthlyWindow(series?.monthly.points ?? [], vehicleId, selection),
  );
  $effect(() => {
    const normalized = window.selection;
    if (
      selection?.vehicleId !== normalized?.vehicleId ||
      selection?.start !== normalized?.start ||
      selection?.followingLatest !== normalized?.followingLatest
    ) {
      selection = normalized;
    }
  });
  const selectedSeries = $derived(
    mode === "monthly" ? { points: window.points } : series?.yearly,
  );
  const chartData = $derived(
    selectedSeries ? toPeriodData(selectedSeries) : [],
  );

  const chartTitle = $derived(
    mode === "monthly"
      ? t("charts.monthlyCost", { currency })
      : t("charts.yearlyCost", { currency }),
  );

  const bandScale = $derived(
    scaleBand<string>()
      .domain(chartData.map((d) => d.month))
      .paddingInner(0.2)
      .paddingOuter(0.1),
  );

  const yFormat = $derived((v: number) =>
    formatCurrency(v, currency, settings.locale),
  );

  const tooltipFormatY = $derived((d: Record<string, unknown>) => {
    const v = d.value as number;
    return formatCurrency(v, currency, settings.locale);
  });

  const tooltipFormatX = $derived((d: Record<string, unknown>) => {
    const m = d.month as string;
    if (mode === "monthly" && /^\d{4}-\d{2}$/.test(m)) {
      const [y, mo] = m.split("-");
      const dt = new Date(Number(y), Number(mo) - 1);
      return dt.toLocaleDateString(settings.locale, {
        month: "long",
        year: "numeric",
      });
    }
    return m;
  });
</script>

<ChartCard
  title={chartTitle}
  data={chartData}
  x="month"
  y="value"
  xScale={bandScale}
  yDomain={[0, null]}
  minimumDataPoints={1}
>
  {#snippet actions()}
    <div class="chart-actions">
      {#if mode === "monthly" && (series?.monthly.points.length ?? 0) > 12}
        <PeriodNavigation
          {window}
          locale={settings.locale}
          onmove={(value) => (selection = value)}
        />
      {/if}
      <div class="segmented segmented-sm">
        <button
          type="button"
          class="segmented-item"
          class:active={mode === "monthly"}
          onclick={() => (mode = "monthly")}
        >
          {t("charts.monthly")}
        </button>
        <button
          type="button"
          class="segmented-item"
          class:active={mode === "yearly"}
          onclick={() => (mode = "yearly")}
        >
          {t("charts.yearly")}
        </button>
      </div>
    </div>
  {/snippet}
  <AxisY format={yFormat} />
  <AxisX />
  <Bar />
  <Tooltip formatY={tooltipFormatY} formatX={tooltipFormatX} />
</ChartCard>

<style>
  .chart-actions {
    display: flex;
    flex-wrap: wrap;
    justify-content: flex-end;
    align-items: center;
  }
  .segmented-sm {
    transform: scale(0.85);
    transform-origin: right center;
  }
</style>
