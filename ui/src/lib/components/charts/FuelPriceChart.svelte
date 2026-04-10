<script lang="ts">
  import { scaleTime } from "d3-scale";
  import type { SegmentHistory } from "$lib/api";
  import { t } from "$lib/i18n";
  import { toFuelPriceData } from "$lib/charts";
  import { formatCurrency } from "$lib/format";
  import { getSettings } from "$lib/stores/settings.svelte";
  import ChartCard from "./ChartCard.svelte";
  import Line from "./Line.svelte";
  import AxisX from "./AxisX.svelte";
  import AxisY from "./AxisY.svelte";
  import Tooltip from "./Tooltip.svelte";

  let {
    segments,
    currency,
    volumeUnit,
  }: {
    segments: SegmentHistory[];
    currency: string;
    volumeUnit: string;
  } = $props();

  const settings = $derived(getSettings());

  const chartData = $derived(toFuelPriceData(segments));

  // Dynamic y-domain: floor below min value with padding
  const yMin = $derived(() => {
    if (chartData.length === 0) return 0;
    const min = Math.min(...chartData.map((d) => d.value));
    const max = Math.max(...chartData.map((d) => d.value));
    const padding = (max - min) * 0.15 || min * 0.1;
    return Math.max(0, Math.floor((min - padding) * 20) / 20);
  });

  const volLabel = $derived(volumeUnit === "l" ? "L" : volumeUnit);

  const yFormat = $derived(
    (v: number) => `${formatCurrency(v, currency, settings.locale)}`,
  );

  const tooltipFormatY = $derived((d: Record<string, unknown>) => {
    const v = d.value as number;
    return `${formatCurrency(v, currency, settings.locale)}/${volLabel}`;
  });
</script>

<ChartCard
  title={t("charts.fuelPrice", { currency, unit: volLabel })}
  data={chartData}
  x={(d: { date: Date }) => d.date}
  y={(d: { value: number }) => d.value}
  xScale={scaleTime()}
  yDomain={[yMin(), null]}
>
  <AxisY format={yFormat} />
  <AxisX />
  <Line />
  <Tooltip formatY={tooltipFormatY} />
</ChartCard>
