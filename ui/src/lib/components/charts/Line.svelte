<script lang="ts">
  import { getContext } from "svelte";
  import type { LayerCakeContext } from "$lib/charts";

  const { data, xGet, yGet } = getContext("LayerCake") as LayerCakeContext;

  let { stroke = "var(--color-accent)", strokeWidth = 2 } = $props();
</script>

<path
  d={`M${$data.map((d) => `${$xGet(d)},${$yGet(d)}`).join("L")}`}
  fill="none"
  {stroke}
  stroke-width={strokeWidth}
  stroke-linejoin="round"
  stroke-linecap="round"
/>
