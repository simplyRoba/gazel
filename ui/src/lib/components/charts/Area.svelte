<script lang="ts">
  import { getContext } from "svelte";
  import type { LayerCakeContext } from "$lib/charts";

  const { data, xGet, yGet, height } = getContext(
    "LayerCake",
  ) as LayerCakeContext;

  let { fill = "var(--color-accent-subtle)", opacity = 0.3 } = $props();
</script>

{#if $data.length >= 2}
  <path
    d={`M${$data.map((d) => `${$xGet(d)},${$yGet(d)}`).join("L")}L${$xGet($data[$data.length - 1])},${$height}L${$xGet($data[0])},${$height}Z`}
    {fill}
    {opacity}
    stroke="none"
  />
{/if}
