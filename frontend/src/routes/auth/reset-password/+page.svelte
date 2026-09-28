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
  <title>{i18n.t('auth.reset_password.page_title')} — Civitas</title>
</svelte:head>

<section class="mx-auto w-full max-w-md px-5 py-16 sm:px-6">
  <div class="mb-3 font-mono text-[11px] uppercase tracking-[0.2em] text-ink-400">Civitas</div>
  <h1 class="font-serif text-[34px] font-semibold leading-[1.1] tracking-[-0.01em]">
    {i18n.t('auth.reset_password.heading')}
  </h1>

  <div class="mt-7 space-y-4">
    {#if errorMessage}
      <Banner tone="error" title={i18n.t('auth.reset_password.error_title')}>{errorMessage}</Banner>
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
        name="token"
        label={i18n.t('auth.reset_password.token_label')}
        required
        value={data.prefilledToken}
        hint={data.prefilledToken
          ? i18n.t('auth.reset_password.token_hint_prefilled')
          : i18n.t('auth.reset_password.token_hint')}
      />
      <TextField
        name="new_password"
        label={i18n.t('auth.reset_password.password_label')}
        type="password"
        required
        autocomplete="new-password"
        minlength={12}
        hint={i18n.t('auth.reset_password.password_hint')}
      />
      <div class="flex flex-wrap items-center justify-between gap-4">
        <Button type="submit" loading={submitting}>{i18n.t('auth.reset_password.submit')}</Button>
        <a href="/auth/forgot-password" class="text-sm text-accent-600 hover:underline">
          {i18n.t('auth.reset_password.new_link')}
        </a>
      </div>
    </form>
  </div>
</section>
