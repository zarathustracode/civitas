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
  let resending = $state(false);

  const errorMessage = $derived(
    form?.code ? friendlyMessage(new ApiError(form.code, form.code, 0), i18n) : null
  );
</script>

<svelte:head>
  <title>{i18n.t('auth.verify_email.page_title')} — Civitas</title>
</svelte:head>

<section class="mx-auto w-full max-w-md px-5 py-16 sm:px-6">
  <div class="mb-3 font-mono text-[11px] uppercase tracking-[0.2em] text-ink-400">Civitas</div>
  <h1 class="font-serif text-[34px] font-semibold leading-[1.1] tracking-[-0.01em]">
    {i18n.t('auth.verify_email.heading')}
  </h1>

  <div class="mt-7 space-y-4">
    {#if data.registered}
      <Banner tone="success" title={i18n.t('auth.verify_email.registered_title')}>
        {i18n.t('auth.verify_email.registered_body')}
      </Banner>
    {/if}
    {#if form?.resent}
      <Banner tone="success" title={i18n.t('auth.verify_email.resent_title')}>
        {i18n.t('auth.verify_email.resent_body')}
      </Banner>
    {/if}
    {#if errorMessage}
      <Banner tone="error" title={i18n.t('auth.verify_email.error_title')}>{errorMessage}</Banner>
    {/if}

    <form
      method="POST"
      action="?/verify"
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
        name="token"
        label={i18n.t('auth.verify_email.token_label')}
        required
        value={data.prefilledToken}
        hint={data.prefilledToken
          ? i18n.t('auth.verify_email.token_hint_prefilled')
          : i18n.t('auth.verify_email.token_hint')}
      />
      <Button type="submit" loading={submitting}>{i18n.t('auth.verify_email.submit')}</Button>
    </form>
  </div>

  <div class="mt-10 border-t border-line pt-8">
    <h2 class="font-serif text-[20px] font-semibold">
      {i18n.t('auth.verify_email.resend_heading')}
    </h2>
    <p class="mt-2 font-serif text-[16px] leading-[1.5] text-ink-600">
      {i18n.t('auth.verify_email.resend_intro')}
    </p>
    <form
      method="POST"
      action="?/resend"
      class="mt-4 flex flex-col gap-4"
      use:enhance={() => {
        resending = true;
        return async ({ update }) => {
          await update();
          resending = false;
        };
      }}
    >
      <TextField
        name="email"
        label={i18n.t('auth.verify_email.email_label')}
        type="email"
        required
        autocomplete="email"
      />
      <Button type="submit" variant="secondary" loading={resending}>
        {i18n.t('auth.verify_email.resend_submit')}
      </Button>
    </form>
  </div>
</section>
