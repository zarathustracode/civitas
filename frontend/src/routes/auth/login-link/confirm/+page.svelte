<script lang="ts">
  import { enhance } from '$app/forms';
  import Button from '$lib/components/Button.svelte';
  import Banner from '$lib/components/Banner.svelte';
  import { friendlyMessage, ApiError } from '$lib/api/errors';
  import { getI18n } from '$lib/i18n';
  import type { ActionData, PageData } from './$types';

  let { data, form }: { data: PageData; form: ActionData } = $props();
  const i18n = getI18n();
  let submitting = $state(false);

  const errorMessage = $derived(
    form?.code ? friendlyMessage(new ApiError(form.code, form.code, 0), i18n) : null
  );
</script>

<svelte:head>
  <title>{i18n.t('auth.login_link_confirm.page_title')} — Civitas</title>
</svelte:head>

<section class="mx-auto w-full max-w-md px-5 py-16 sm:px-6">
  <div class="mb-3 font-mono text-[11px] uppercase tracking-[0.2em] text-ink-400">Civitas</div>
  <h1 class="font-serif text-[34px] font-semibold leading-[1.1] tracking-[-0.01em]">
    {i18n.t('auth.login_link_confirm.heading')}
  </h1>

  <div class="mt-7 space-y-4">
    {#if !data.token}
      <Banner tone="error" title={i18n.t('auth.login_link_confirm.missing_title')}>
        {i18n.t('auth.login_link_confirm.missing_body')}
        <a href="/auth/login-link" class="font-medium underline"
          >{i18n.t('auth.login_link_confirm.request_new')}</a
        >
      </Banner>
    {:else}
      {#if errorMessage}
        <Banner tone="error" title={i18n.t('auth.login_link_confirm.error_title')}>
          {errorMessage}
          {i18n.t('auth.login_link_confirm.error_hint')}
          <a href="/auth/login-link" class="font-medium underline"
            >{i18n.t('auth.login_link_confirm.request_new')}</a
          >
        </Banner>
      {:else}
        <p class="font-serif text-[17px] leading-[1.5] text-ink-600">
          {i18n.t('auth.login_link_confirm.intro')}
        </p>
      {/if}

      <form
        method="POST"
        use:enhance={() => {
          submitting = true;
          return async ({ update }) => {
            await update();
            submitting = false;
          };
        }}
      >
        <input type="hidden" name="token" value={data.token} />
        <Button type="submit" loading={submitting}>
          {i18n.t('auth.login_link_confirm.submit')}
        </Button>
      </form>
    {/if}
  </div>
</section>
