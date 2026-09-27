<script lang="ts">
  import { enhance } from '$app/forms';
  import Button from '$lib/components/Button.svelte';
  import TextField from '$lib/components/TextField.svelte';
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
  <title>{i18n.t('auth.login.page_title')} — Civitas</title>
</svelte:head>

<section class="mx-auto w-full max-w-md px-5 py-16 sm:px-6">
  <div class="mb-3 font-mono text-[11px] uppercase tracking-[0.2em] text-ink-400">Civitas</div>
  <h1 class="font-serif text-[34px] font-semibold leading-[1.1] tracking-[-0.01em]">
    {i18n.t('auth.login.heading')}
  </h1>

  <div class="mt-7 space-y-4">
    {#if data.verified}
      <Banner tone="success" title={i18n.t('auth.login.verified_title')}>
        {i18n.t('auth.login.verified_body')}
      </Banner>
    {/if}
    {#if data.reset}
      <Banner tone="success" title={i18n.t('auth.login.reset_title')}>
        {i18n.t('auth.login.reset_body')}
      </Banner>
    {/if}
    {#if errorMessage}
      <Banner tone="error" title={i18n.t('auth.login.error_title')}>{errorMessage}</Banner>
    {/if}

    <form
      method="POST"
      class="flex flex-col gap-4"
      use:enhance={() => {
        submitting = true;
        return async ({ update }) => {
          await update();
          submitting = false;
        };
      }}
    >
      <TextField
        name="email"
        label={i18n.t('auth.login.email_label')}
        type="email"
        required
        autocomplete="email"
        value={form?.email ?? ''}
      />
      <TextField
        name="password"
        label={i18n.t('auth.login.password_label')}
        type="password"
        required
        autocomplete="current-password"
        minlength={12}
      />
      <div class="flex flex-wrap items-center justify-between gap-4">
        <Button type="submit" loading={submitting}>{i18n.t('auth.login.submit')}</Button>
        <div class="flex gap-4 text-sm">
          <a href="/auth/forgot-password" class="text-accent-600 hover:underline">
            {i18n.t('auth.login.forgot_password')}
          </a>
          <a href="/auth/register" class="text-accent-600 hover:underline">
            {i18n.t('auth.login.no_account')}
          </a>
        </div>
      </div>
    </form>

    <p class="border-t border-line pt-5 text-sm text-ink-600">
      {i18n.t('auth.login.link_lead')}
      <a
        href="/auth/login-link"
        class="text-accent-600 underline underline-offset-2 hover:text-accent-700"
        >{i18n.t('auth.login.link_action')}</a
      >
    </p>
  </div>
</section>
