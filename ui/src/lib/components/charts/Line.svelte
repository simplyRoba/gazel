<script lang="ts">
  import { getContext } from "svelte";
  import { line, curveMonotoneX } from "d3-shape";
  import type { LayerCakeContext } from "$lib/charts";

  const { data, xGet, yGet } = getContext("LayerCake") as LayerCakeContext;

  let { stroke = "var(--color-accent)", strokeWidth = 2 } = $props();

  /* eslint-disable @typescript-eslint/no-explicit-any */
  const pathD = $derived(
    line<Record<string, any>>()
      .x((d) => $xGet(d))
      .y((d) => $yGet(d))
      .curve(curveMonotoneX)($data) ?? "",
  );
  /* eslint-enable @typescript-eslint/no-explicit-any */
</script>

<path
  d={pathD}
  fill="none"
  {stroke}
  stroke-width={strokeWidth}
  stroke-linejoin="round"
  stroke-linecap="round"
/>
