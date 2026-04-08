<script lang="ts">
  import PageContainer from "$lib/components/PageContainer.svelte";
</script>

<PageContainer>
  <h1 class="page-title">Settings</h1>
  <p class="page-subtitle">Configure vehicles, units, appearance, and data.</p>
</PageContainer>

<style>
  .page-title {
    font-size: var(--font-2xl);
    font-weight: var(--font-weight-bold);
    margin-bottom: var(--space-2);
  }

  .page-subtitle {
    font-size: var(--font-md);
    color: var(--color-text-secondary);
  }
</style>
