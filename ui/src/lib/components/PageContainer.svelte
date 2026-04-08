<script lang="ts">
  import type { Snippet } from "svelte";

  let {
    width = "default",
    children,
  }: {
    width?: "narrow" | "default" | "wide";
    children?: Snippet;
  } = $props();
</script>

<div
  class="page-container"
  class:narrow={width === "narrow"}
  class:wide={width === "wide"}
>
  {@render children?.()}
</div>

<style>
  .page-container {
    max-width: var(--content-width-default);
    margin: 0 auto;
    width: 100%;
  }

  .narrow {
    max-width: var(--content-width-narrow);
  }

  .wide {
    max-width: var(--content-width-wide);
  }
</style>
