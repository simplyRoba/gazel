<script lang="ts">
  import { scalePoint } from "d3-scale";
  import type { PeriodModes } from "#lib/api.js";
  import { t } from "#lib/i18n/index.js";
  import { toPeriodData } from "#lib/charts.js";
  import { monthlyWindow, type MonthlySelection } from "#lib/monthly-window.js";
  import PeriodNavigation from "./PeriodNavigation.svelte";
  import { formatDistance } from "#lib/format.js";
  import { getSettings } from "#lib/stores/settings.svelte.js";
  import ChartCard from "./ChartCard.svelte";
  import Line from "./Line.svelte";
  import Area from "./Area.svelte";
  import AxisX from "./AxisX.svelte";
  import AxisY from "./AxisY.svelte";
  import Tooltip from "./Tooltip.svelte";

  let {
    series,
    distanceUnit,
    vehicleId = null,
    selection = $bindable(null),
  }: {
    series: PeriodModes | null;
    distanceUnit: string;
    vehicleId?: number | null;
    selection?: MonthlySelection | null;
  } = $props();

  const settings = $derived(getSettings());

  type DistanceMode = "monthly" | "yearly";
  let mode: DistanceMode = $state("monthly");

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
      ? t("charts.monthlyDistance", { unit: distanceUnit })
      : t("charts.yearlyDistance", { unit: distanceUnit }),
  );

  const pointScale = $derived(
    scalePoint<string>()
      .domain(chartData.map((d) => d.month))
      .padding(0.1),
  );

  const yFormat = $derived((v: number) => {
    const num = new Intl.NumberFormat(settings.locale, {
      maximumFractionDigits: 0,
    }).format(v);
    return `${num} ${distanceUnit}`;
  });

  const tooltipFormatY = $derived((d: Record<string, unknown>) => {
    const v = d.value as number;
    return formatDistance(v, distanceUnit, settings.locale);
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
  xScale={pointScale}
  yDomain={[0, null]}
>
  {#snippet actions()}
    <div class="chart-actions">
      {#if mode === "monthly" && (series?.monthly.points.length ?? 0) > 12}
        <PeriodNavigation {window} onmove={(value) => (selection = value)} />
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
  <Area />
  <Line />
  <Tooltip formatY={tooltipFormatY} formatX={tooltipFormatX} />
</ChartCard>

<style>
  .chart-actions {
    display: flex;
    flex-wrap: wrap;
    justify-content: flex-end;
    align-items: center;
  }
</style>
