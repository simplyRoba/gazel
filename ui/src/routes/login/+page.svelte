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
          config = value;
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
    <section
      class="login-card login-card-error corner-tri"
      aria-labelledby="login-title"
    >
      <div class="brand" aria-hidden="true"><Logo size={64} /></div>
      <h1 id="login-title">{t("login.title")}</h1>
      <p role="alert">{t("login.error.configUnavailable")}</p>
    </section>
  </main>
{:else if config === null || config.enabled}
  <main class="login-page">
    <section
      class="login-card corner-tri"
      aria-labelledby="login-title"
      aria-busy={config === null}
    >
      <div class="login-copy">
        <div class="brand" aria-hidden="true"><Logo size={80} /></div>
        <h1 id="login-title">{t("login.title")}</h1>
        <p class="intro">{t("login.authenticationRequired")}</p>
      </div>

      <div class="login-action-panel">
        <div class="login-panel">
          {#if config === null}
            <p class="loading-status" role="status">{t("common.loading")}</p>
          {:else}
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
          {/if}
        </div>
      </div>
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
    overflow: hidden;
    text-align: center;
    background: var(--color-bg-raised);
    border: 1px solid var(--color-border);
    box-shadow: var(--shadow-lg);
  }

  .login-copy {
    padding: var(--space-8) var(--space-8) 0;
  }

  .login-action-panel {
    padding: 0 var(--space-8) var(--space-8);
  }

  .login-card-error {
    padding: var(--space-8);
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
    color: var(--color-error);
    background: color-mix(in srgb, var(--color-error) 8%, transparent);
    border: 1px solid color-mix(in srgb, var(--color-error) 20%, transparent);
  }

  .status.success {
    color: var(--color-success);
    background: color-mix(in srgb, var(--color-success) 8%, transparent);
    border-color: color-mix(in srgb, var(--color-success) 20%, transparent);
  }

  .loading-status {
    margin: 0;
    color: var(--color-text-secondary);
    text-align: center;
  }

  form {
    width: 100%;
  }

  .login-action {
    width: 100%;
    justify-content: center;
  }

  @media (min-width: 48rem) {
    .login-page {
      padding: var(--space-8);
    }

    .login-card {
      width: min(100%, 880px);
      min-height: 380px;
      display: grid;
      grid-template-columns: minmax(0, 1.15fr) minmax(300px, 0.85fr);
      text-align: left;
    }

    .login-copy {
      display: flex;
      flex-direction: column;
      justify-content: center;
      align-items: flex-start;
      padding: var(--space-12);
      background: var(--color-bg-feature);
      border-right: 1px solid var(--color-border-feature);
    }

    .login-action-panel {
      display: flex;
      align-items: center;
      padding: var(--space-10);
    }

    .login-panel {
      width: 100%;
      padding: var(--space-6);
      background: var(--color-bg-raised);
      border: 1px solid var(--color-border);
      box-shadow: var(--shadow-md);
    }

    .login-card-error {
      width: min(100%, 560px);
      min-height: auto;
      display: block;
      padding: var(--space-10);
      text-align: center;
    }

    .brand {
      margin: 0 0 var(--space-5);
    }

    h1 {
      font-size: 2.5rem;
    }

    .intro {
      max-width: 28rem;
      margin-bottom: 0;
      font-size: var(--font-lg);
    }

    .login-card-error .brand {
      margin: 0 auto var(--space-4);
    }
  }
</style>
