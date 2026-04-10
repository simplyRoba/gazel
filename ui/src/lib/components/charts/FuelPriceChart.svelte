<script lang="ts">
  import { scaleTime } from "d3-scale";
  import type { SegmentHistory } from "$lib/api";
  import { toFuelPriceData } from "$lib/charts";
  import { formatCurrency } from "$lib/format";
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

  const chartData = $derived(toFuelPriceData(segments));

  const volLabel = $derived(volumeUnit === "l" ? "L" : volumeUnit);

  const yFormat = $derived((v: number) => `${formatCurrency(v, currency)}`);

  const tooltipFormatY = $derived((d: Record<string, unknown>) => {
    const v = d.value as number;
    return `${formatCurrency(v, currency)}/${volLabel}`;
  });
</script>

<ChartCard
  title="Fuel price ({currency}/{volLabel})"
  data={chartData}
  x={(d: { date: Date }) => d.date}
  y={(d: { value: number }) => d.value}
  xScale={scaleTime()}
  yDomain={[0, null]}
>
  <AxisY format={yFormat} />
  <AxisX />
  <Line />
  <Tooltip formatY={tooltipFormatY} />
</ChartCard>
