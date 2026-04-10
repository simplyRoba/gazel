<script lang="ts">
  import { scaleBand } from "d3-scale";
  import type { SegmentHistory } from "$lib/api";
  import { t } from "$lib/i18n";
  import { toMonthlyCostData } from "$lib/charts";
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

  const MAX_MONTHS = 12;
  const chartData = $derived(toMonthlyCostData(segments).slice(-MAX_MONTHS));

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
    if (/^\d{4}-\d{2}$/.test(m)) {
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
  title={t("charts.monthlyCost", { currency })}
  data={chartData}
  x="month"
  y="value"
  xScale={bandScale}
  yDomain={[0, null]}
>
  <AxisY format={yFormat} />
  <AxisX />
  <Bar />
  <Tooltip formatY={tooltipFormatY} formatX={tooltipFormatX} />
</ChartCard>
