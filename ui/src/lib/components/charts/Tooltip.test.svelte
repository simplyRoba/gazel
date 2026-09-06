<script lang="ts">
  import { setContext } from "svelte";
  import { readable } from "svelte/store";
  import { scaleLinear } from "d3-scale";

  import type { LayerCakeContext } from "$lib/charts";

  import Tooltip from "./Tooltip.svelte";

  const points = [
    { x: 20, y: 30, date: "First", value: 1 },
    { x: 80, y: 10, date: "Second", value: 2 },
  ];

  setContext<LayerCakeContext>("LayerCake", {
    data: readable(points),
    xGet: readable((point) => Number(point.x)),
    yGet: readable((point) => Number(point.y)),
    xScale: readable(scaleLinear()),
    yScale: readable(scaleLinear()),
    width: readable(100),
    height: readable(50),
  });
</script>

<svg>
  <Tooltip
    formatX={(point: Record<string, unknown>) => String(point.date)}
    formatY={(point: Record<string, unknown>) => String(point.value)}
  />
</svg>
