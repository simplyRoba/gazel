<script lang="ts">
  import { scaleTime } from "d3-scale";
  import type { SegmentHistory } from "$lib/api";
  import { t } from "$lib/i18n";
  import { getSettings } from "$lib/stores/settings.svelte";
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

  const settings = $derived(getSettings());
  const unitLabel = $derived(efficiencyUnitLabel(distanceUnit, volumeUnit));

  // Dynamic y-domain: floor at ~90% of min value, rounded down
  const yMin = $derived(() => {
    if (chartData.length === 0) return 0;
    const min = Math.min(...chartData.map((d) => d.value));
    const max = Math.max(...chartData.map((d) => d.value));
    const padding = (max - min) * 0.15 || min * 0.1;
    return Math.max(0, Math.floor((min - padding) * 2) / 2);
  });

  function fmtNum(v: number): string {
    return new Intl.NumberFormat(settings.locale, {
      minimumFractionDigits: 1,
      maximumFractionDigits: 1,
    }).format(v);
  }

  const yFormat = $derived((v: number) => fmtNum(v));

  const tooltipFormatY = $derived((d: Record<string, unknown>) => {
    const v = d.value as number;
    const unit = efficiencyUnitLabel(distanceUnit, volumeUnit);
    return `${fmtNum(v)} ${unit}`;
  });
</script>

<ChartCard
  title={t("charts.efficiency", { unit: unitLabel })}
  data={chartData}
  x={(d: { date: Date }) => d.date}
  y={(d: { value: number }) => d.value}
  xScale={scaleTime()}
  yDomain={[yMin(), null]}
>
  <AxisY format={yFormat} />
  <AxisX />
  <Area />
  <Line />
  <Tooltip formatY={tooltipFormatY} />
</ChartCard>
