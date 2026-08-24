<script lang="ts">
  import { onMount } from "svelte";
  import { page } from "$app/state";
  import { fetchAuthConfig, type AuthConfig } from "$lib/api";
  import Logo from "$lib/components/Logo.svelte";
  import { t } from "$lib/i18n";

  let config = $state<AuthConfig | null>(null);
  let configFailed = $state(false);

  let returnTo = $derived(page.url.searchParams.get("return_to") ?? "/");
  let error = $derived(page.url.searchParams.get("error"));
  let loggedOut = $derived(page.url.searchParams.get("logged_out") === "1");
  let statusMessage = $derived(
    error === "provider_unavailable"
      ? t("login.error.providerUnavailable")
      : error !== null
        ? t("login.error.authenticationFailed")
        : loggedOut
          ? t("login.loggedOut")
          : null,
  );

  onMount(() => {
    const controller = new AbortController();

    void fetchAuthConfig(controller.signal)
      .then((value) => {
        if (controller.signal.aborted) return;
        if (!value.enabled) {
          window.location.replace("/");
          return;
        }
        config = value;
      })
      .catch(() => {
        if (!controller.signal.aborted) configFailed = true;
      });

    return () => controller.abort();
  });
</script>

{#if configFailed}
  <main class="login-page">
    <section class="login-card corner-tri" aria-labelledby="login-title">
      <div class="brand" aria-hidden="true"><Logo size={64} /></div>
      <h1 id="login-title">{t("login.title")}</h1>
      <p role="alert">{t("login.error.configUnavailable")}</p>
    </section>
  </main>
{:else if config?.enabled}
  <main class="login-page">
    <section class="login-card corner-tri" aria-labelledby="login-title">
      <div class="brand" aria-hidden="true"><Logo size={64} /></div>
      <h1 id="login-title">{t("login.title")}</h1>
      <p class="intro">{t("login.authenticationRequired")}</p>

      {#if statusMessage}
        <p class:success={loggedOut} class="status" role="alert">
          {statusMessage}
        </p>
      {/if}

      <form method="GET" action="/auth/login">
        <input name="return_to" type="hidden" value={returnTo} />
        <button class="btn btn-primary login-action" type="submit">
          {t("login.continueWith", { provider: config.provider_name })}
        </button>
      </form>
    </section>
  </main>
{/if}

<style>
  .login-page {
    min-height: 100dvh;
    display: grid;
    place-items: center;
    padding: var(--space-4);
    background: var(--color-bg);
  }

  .login-card {
    width: min(100%, 400px);
    padding: var(--space-8);
    text-align: center;
    background: var(--color-bg-raised);
    border: 1px solid var(--color-border);
    box-shadow: var(--shadow-lg);
  }

  .brand {
    color: var(--color-brand-gold-1);
    margin: 0 auto var(--space-4);
    width: fit-content;
  }

  h1 {
    font-size: var(--font-2xl);
    font-weight: var(--font-weight-bold);
    margin-bottom: var(--space-3);
  }

  .intro {
    color: var(--color-text-secondary);
    margin-bottom: var(--space-6);
  }

  .status {
    margin: 0 0 var(--space-5);
    padding: var(--space-3);
    color: var(--color-text-danger, #dc2626);
    background: var(--color-surface-danger, rgba(220, 38, 38, 0.08));
    border: 1px solid var(--color-border-danger, rgba(220, 38, 38, 0.2));
  }

  .status.success {
    color: var(--color-text-success, #15803d);
  }

  form {
    width: 100%;
  }

  .login-action {
    width: 100%;
    justify-content: center;
  }
</style>
