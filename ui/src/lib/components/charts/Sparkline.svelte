<script lang="ts">
  /**
   * Standalone sparkline — does NOT require a LayerCake parent.
   * Renders a minimal area+line chart as an inline SVG background.
   */

  import type { SparklinePoint } from "$lib/charts";

  let {
    data = [] as SparklinePoint[],
    stroke = "var(--color-accent)",
    fill = "var(--color-accent)",
    fillOpacity = 0.15,
    strokeWidth = 1.5,
  } = $props();

  function computePaths(points: SparklinePoint[]): {
    line: string;
    area: string;
  } {
    if (points.length < 2) return { line: "", area: "" };

    const xMin = Math.min(...points.map((p) => p.x));
    const xMax = Math.max(...points.map((p) => p.x));
    const yMin = Math.min(...points.map((p) => p.y));
    const yMax = Math.max(...points.map((p) => p.y));

    const xRange = xMax - xMin || 1;
    const yRange = yMax - yMin || 1;

    const mapped = points.map((p) => ({
      sx: ((p.x - xMin) / xRange) * 100,
      sy: 100 - ((p.y - yMin) / yRange) * 80 - 10, // 10-90 range for padding
    }));

    const linePoints = mapped.map((p) => `${p.sx},${p.sy}`).join(" L");
    const line = `M${linePoints}`;
    const area = `${line} L${mapped[mapped.length - 1].sx},100 L${mapped[0].sx},100 Z`;

    return { line, area };
  }

  const paths = $derived(computePaths(data));
</script>

<svg
  viewBox="0 0 100 100"
  preserveAspectRatio="none"
  class="sparkline-svg"
  aria-hidden="true"
>
  {#if paths.area}
    <path d={paths.area} {fill} opacity={fillOpacity} stroke="none" />
    <path
      d={paths.line}
      fill="none"
      {stroke}
      stroke-width={strokeWidth}
      stroke-linejoin="round"
      vector-effect="non-scaling-stroke"
    />
  {/if}
</svg>

<style>
  .sparkline-svg {
    width: 100%;
    height: 100%;
    display: block;
  }
</style>
