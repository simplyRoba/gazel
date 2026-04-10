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

    // Build smooth cubic Bezier curve (Catmull-Rom → cubic control points)
    let line = `M${mapped[0].sx},${mapped[0].sy}`;
    const t = 0.3; // tension (0 = sharp, 1 = very smooth)

    for (let i = 0; i < mapped.length - 1; i++) {
      const p0 = mapped[Math.max(0, i - 1)];
      const p1 = mapped[i];
      const p2 = mapped[i + 1];
      const p3 = mapped[Math.min(mapped.length - 1, i + 2)];

      const cp1x = p1.sx + ((p2.sx - p0.sx) * t) / 3;
      const cp1y = p1.sy + ((p2.sy - p0.sy) * t) / 3;
      const cp2x = p2.sx - ((p3.sx - p1.sx) * t) / 3;
      const cp2y = p2.sy - ((p3.sy - p1.sy) * t) / 3;

      line += ` C${cp1x},${cp1y} ${cp2x},${cp2y} ${p2.sx},${p2.sy}`;
    }

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
