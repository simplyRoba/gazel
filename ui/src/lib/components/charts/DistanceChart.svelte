<script lang="ts">
  import { scalePoint } from "d3-scale";
  import type { SegmentHistory } from "$lib/api";
  import { t } from "$lib/i18n";
  import { toMonthlyDistanceData, toYearlyDistanceData } from "$lib/charts";
  import { formatDistance } from "$lib/format";
  import { getSettings } from "$lib/stores/settings.svelte";
  import ChartCard from "./ChartCard.svelte";
  import Line from "./Line.svelte";
  import Area from "./Area.svelte";
  import AxisX from "./AxisX.svelte";
  import AxisY from "./AxisY.svelte";
  import Tooltip from "./Tooltip.svelte";

  let {
    segments,
    distanceUnit,
  }: {
    segments: SegmentHistory[];
    distanceUnit: string;
  } = $props();

  const settings = $derived(getSettings());

  type DistanceMode = "monthly" | "yearly";
  let mode: DistanceMode = $state("monthly");

  const MAX_MONTHS = 12;

  const chartData = $derived(
    mode === "monthly"
      ? toMonthlyDistanceData(segments).slice(-MAX_MONTHS)
      : toYearlyDistanceData(segments),
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
  {/snippet}
  <AxisY format={yFormat} />
  <AxisX />
  <Area />
  <Line />
  <Tooltip formatY={tooltipFormatY} formatX={tooltipFormatX} />
</ChartCard>

<style>
  .segmented-sm {
    transform: scale(0.85);
    transform-origin: right center;
  }
</style>
