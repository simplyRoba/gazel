<script lang="ts">
  import { scaleBand } from "d3-scale";
  import type { SegmentHistory } from "$lib/api";
  import { t } from "$lib/i18n";
  import { toMonthlyCostData, toYearlyCostData } from "$lib/charts";
  import { formatCurrency } from "$lib/format";
  import { getSettings } from "$lib/stores/settings.svelte";
  import ChartCard from "./ChartCard.svelte";
  import Bar from "./Bar.svelte";
  import AxisX from "./AxisX.svelte";
  import AxisY from "./AxisY.svelte";
  import Tooltip from "./Tooltip.svelte";

  let {
    segments,
    currency,
  }: {
    segments: SegmentHistory[];
    currency: string;
  } = $props();

  const settings = $derived(getSettings());

  type CostMode = "monthly" | "yearly";
  let mode: CostMode = $state("monthly");

  const MAX_MONTHS = 12;

  const chartData = $derived(
    mode === "monthly"
      ? toMonthlyCostData(segments).slice(-MAX_MONTHS)
      : toYearlyCostData(segments),
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
  <Bar />
  <Tooltip formatY={tooltipFormatY} formatX={tooltipFormatX} />
</ChartCard>

<style>
  .segmented-sm {
    transform: scale(0.85);
    transform-origin: right center;
  }
</style>
