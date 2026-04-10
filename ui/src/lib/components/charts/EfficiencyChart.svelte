<script lang="ts">
  import { scaleTime } from "d3-scale";
  import type { SegmentHistory } from "$lib/api";
  import { toEfficiencyData } from "$lib/charts";
  import { toDisplayEfficiency, efficiencyUnitLabel } from "$lib/format";
  import ChartCard from "./ChartCard.svelte";
  import Line from "./Line.svelte";
  import Area from "./Area.svelte";
  import AxisX from "./AxisX.svelte";
  import AxisY from "./AxisY.svelte";
  import Tooltip from "./Tooltip.svelte";

  let {
    segments,
    distanceUnit,
    volumeUnit,
  }: {
    segments: SegmentHistory[];
    distanceUnit: string;
    volumeUnit: string;
  } = $props();

  const chartData = $derived(
    toEfficiencyData(segments).map((p) => ({
      ...p,
      value: toDisplayEfficiency(p.value, distanceUnit, volumeUnit),
    })),
  );

  const unitLabel = $derived(efficiencyUnitLabel(distanceUnit, volumeUnit));

  const yFormat = $derived((v: number) => v.toFixed(1));

  const tooltipFormatY = $derived((d: Record<string, unknown>) => {
    const v = d.value as number;
    const unit = efficiencyUnitLabel(distanceUnit, volumeUnit);
    return `${v.toFixed(1)} ${unit}`;
  });
</script>

<ChartCard
  title="Efficiency ({unitLabel})"
  data={chartData}
  x={(d: { date: Date }) => d.date}
  y={(d: { value: number }) => d.value}
  xScale={scaleTime()}
  yDomain={[0, null]}
>
  <AxisY format={yFormat} />
  <AxisX />
  <Area />
  <Line />
  <Tooltip formatY={tooltipFormatY} />
</ChartCard>
