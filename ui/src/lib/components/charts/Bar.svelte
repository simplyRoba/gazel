<script lang="ts">
  import { getContext } from "svelte";
  import type { LayerCakeContext } from "$lib/charts";
  import type { ScaleBand } from "d3-scale";

  const { data, xGet, yGet, xScale, height } = getContext(
    "LayerCake",
  ) as LayerCakeContext;

  let { fill = "var(--color-accent)" } = $props();

  const bandwidth = $derived(
    "bandwidth" in $xScale ? ($xScale as ScaleBand<string>).bandwidth() : 20,
  );
</script>

<g class="bars">
  {#each $data as d, i (i)}
    {@const x = $xGet(d)}
    {@const y = $yGet(d)}
    {@const barHeight = $height - y}
    {#if barHeight > 0}
      <rect {x} {y} width={bandwidth} height={barHeight} {fill} />
    {/if}
  {/each}
</g>
