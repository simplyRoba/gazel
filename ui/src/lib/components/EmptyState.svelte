<script lang="ts">
  import type { Snippet } from "svelte";

  /* eslint-disable @typescript-eslint/no-explicit-any */
  let {
    icon,
    heading,
    description,
    action,
  }: {
    icon?: any;
    heading: string;
    description: string;
    action?: Snippet;
  } = $props();
  /* eslint-enable @typescript-eslint/no-explicit-any */
</script>

<div class="empty-state">
  {#if icon}
    {@const Icon = icon}
    <div class="empty-icon">
      <Icon size={48} />
    </div>
  {/if}
  <h2 class="empty-heading">{heading}</h2>
  <p class="empty-description">{description}</p>
  {#if action}
    <div class="empty-action">
      {@render action()}
    </div>
  {/if}
</div>

<style>
  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    text-align: center;
    padding: var(--space-16) var(--space-4);
    gap: var(--space-3);
  }

  .empty-icon {
    color: var(--color-text-tertiary);
    margin-bottom: var(--space-2);
  }

  .empty-heading {
    font-size: var(--font-lg);
    font-weight: var(--font-weight-semibold);
    color: var(--color-text);
  }

  .empty-description {
    font-size: var(--font-sm);
    color: var(--color-text-secondary);
    max-width: 320px;
    line-height: var(--line-height-normal);
  }

  .empty-action {
    margin-top: var(--space-4);
  }
</style>
