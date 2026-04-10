<script lang="ts">
  import { getContext } from "svelte";
  import type { LayerCakeContext } from "$lib/charts";

  const { data, xGet, yGet, xScale, width, height } = getContext(
    "LayerCake",
  ) as LayerCakeContext;

  // Half-bandwidth offset for band scales (bar charts); 0 for continuous scales
  const bandOffset = $derived(
    "bandwidth" in $xScale && typeof $xScale.bandwidth === "function"
      ? ($xScale as { bandwidth: () => number }).bandwidth() / 2
      : 0,
  );

  let {
    formatX = (d: Record<string, unknown>) => {
      const date = d.date ?? d.month;
      if (date instanceof Date) {
        return date.toLocaleDateString("en", {
          month: "short",
          day: "numeric",
          year: "numeric",
        });
      }
      return String(date ?? "");
    },
    formatY = (d: Record<string, unknown>) => {
      const v = d.value;
      return typeof v === "number" ? v.toFixed(1) : String(v ?? "");
    },
  } = $props();

  let hoveredIndex = $state<number | null>(null);

  function handleMouseMove(event: MouseEvent) {
    const target = event.currentTarget as SVGRectElement;
    const rect = target.getBoundingClientRect();
    const mouseX = event.clientX - rect.left;

    // Find nearest data point (use center of band for bar charts)
    let nearest = 0;
    let nearestDist = Infinity;
    for (let i = 0; i < $data.length; i++) {
      const dist = Math.abs($xGet($data[i]) + bandOffset - mouseX);
      if (dist < nearestDist) {
        nearestDist = dist;
        nearest = i;
      }
    }
    hoveredIndex = nearest;
  }

  function handleMouseLeave() {
    hoveredIndex = null;
  }
</script>

<!-- Invisible overlay to capture mouse events -->
<rect
  x={0}
  y={0}
  width={$width}
  height={$height}
  fill="transparent"
  onmousemove={handleMouseMove}
  onmouseleave={handleMouseLeave}
  ontouchmove={(e) => {
    const touch = e.touches[0];
    handleMouseMove({
      currentTarget: e.currentTarget,
      clientX: touch.clientX,
      clientY: touch.clientY,
    } as unknown as MouseEvent);
  }}
  ontouchend={handleMouseLeave}
/>

{#if hoveredIndex !== null}
  {@const d = $data[hoveredIndex]}
  {@const x = $xGet(d) + bandOffset}
  {@const y = $yGet(d)}

  <!-- Vertical indicator line -->
  <line
    x1={x}
    x2={x}
    y1={0}
    y2={$height}
    stroke="var(--color-text-tertiary)"
    stroke-width={1}
    stroke-dasharray="3,3"
    opacity="0.6"
  />

  <!-- Highlight dot -->
  <circle cx={x} cy={y} r={4} fill="var(--color-accent)" />
  <circle cx={x} cy={y} r={6} fill="var(--color-accent)" opacity="0.3" />

  <!-- Tooltip text pinned to top-right of chart -->
  <text
    x={$width}
    y={-2}
    text-anchor="end"
    fill="var(--color-text)"
    font-size="12"
    font-family="var(--font-family)"
    font-weight="600"
  >
    {formatY(d)}
  </text>
  <text
    x={$width}
    y={12}
    text-anchor="end"
    fill="var(--color-text-secondary)"
    font-size="10"
    font-family="var(--font-family)"
  >
    {formatX(d)}
  </text>
{/if}
