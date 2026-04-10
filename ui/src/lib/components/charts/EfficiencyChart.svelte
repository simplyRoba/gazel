<script lang="ts">
  import { scaleTime } from "d3-scale";
  import type { SegmentHistory } from "$lib/api";
  import { toEfficiencyData } from "$lib/charts";
  import { formatEfficiency } from "$lib/format";
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

  const chartData = $derived(toEfficiencyData(segments));

  const unitLabel = $derived(
    distanceUnit === "mi" && volumeUnit === "gal"
      ? "mpg"
      : `${distanceUnit}/${volumeUnit === "l" ? "L" : volumeUnit}`,
  );

  const yFormat = $derived((v: number) => v.toFixed(1));

  const tooltipFormatY = $derived((d: Record<string, unknown>) => {
    const v = d.value as number;
    return formatEfficiency(v, distanceUnit, volumeUnit);
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
