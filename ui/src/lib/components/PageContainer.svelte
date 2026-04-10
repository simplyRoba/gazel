<script lang="ts">
  import type { Snippet } from "svelte";

  let {
    width = "default",
    fillHeight = false,
    children,
  }: {
    width?: "narrow" | "default" | "wide";
    fillHeight?: boolean;
    children?: Snippet;
  } = $props();
</script>

<div
  class="page-container"
  class:narrow={width === "narrow"}
  class:wide={width === "wide"}
  class:fill-height={fillHeight}
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

  @media (min-width: 960px) {
    .fill-height {
      display: flex;
      flex-direction: column;
      height: calc(100dvh - var(--space-6) * 2);
      overflow: hidden;
    }
  }

  @media (min-width: 1280px) {
    .fill-height {
      height: calc(100dvh - var(--space-8) * 2);
    }
  }
</style>
