<script lang="ts">
  import { LogOut } from "lucide-svelte";
  import { t } from "$lib/i18n";

  let { authEnabled }: { authEnabled: boolean } = $props();
</script>

{#if authEnabled}
  <section class="section corner-tri corner-tri-sm">
    <h2 class="section-title">
      <LogOut size={14} />
      {t("settings.authentication")}
    </h2>
    <div class="setting-row">
      <div class="data-row-info">
        <span class="setting-label">{t("settings.authentication")}</span>
        <span class="data-description"
          >{t("settings.authentication.description")}</span
        >
      </div>
      <form method="POST" action="/auth/logout">
        <button class="btn btn-secondary" type="submit">
          <LogOut size={14} />
          {t("settings.authentication.signOut")}
        </button>
      </form>
    </div>
  </section>
{/if}

<style>
  .setting-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--space-3) 0;
  }

  .data-row-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .setting-label {
    font-size: var(--font-sm);
    font-weight: var(--font-weight-medium);
    color: var(--color-text);
  }

  .data-description {
    font-size: var(--font-xs);
    color: var(--color-text-tertiary);
  }

  @media (max-width: 768px) {
    .setting-row {
      flex-direction: column;
      align-items: flex-start;
      gap: var(--space-2);
    }
  }
</style>
