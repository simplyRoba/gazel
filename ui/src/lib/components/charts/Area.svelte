<script lang="ts">
  import { getContext } from "svelte";
  import { area, curveMonotoneX } from "d3-shape";
  import type { LayerCakeContext } from "$lib/charts";

  const { data, xGet, yGet, height } = getContext(
    "LayerCake",
  ) as LayerCakeContext;

  let { fill = "var(--color-accent-subtle)", opacity = 0.3 } = $props();

  /* eslint-disable @typescript-eslint/no-explicit-any */
  const pathD = $derived(
    area<Record<string, any>>()
      .x((d) => $xGet(d))
      .y0($height)
      .y1((d) => $yGet(d))
      .curve(curveMonotoneX)($data) ?? "",
  );
  /* eslint-enable @typescript-eslint/no-explicit-any */
</script>

{#if $data.length >= 2}
  <path d={pathD} {fill} {opacity} stroke="none" />
{/if}
