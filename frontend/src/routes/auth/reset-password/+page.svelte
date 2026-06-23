<script lang="ts">
  import { enhance } from '$app/forms';
  import Button from '$lib/components/Button.svelte';
  import TextField from '$lib/components/TextField.svelte';
  import Banner from '$lib/components/Banner.svelte';
  import { friendlyMessage, ApiError } from '$lib/api/errors';
  import type { ActionData, PageData } from './$types';

  let { data, form }: { data: PageData; form: ActionData } = $props();
  let submitting = $state(false);

  const errorMessage = $derived(
    form?.code ? friendlyMessage(new ApiError(form.code, form.code, 0)) : null
  );
</script>

<svelte:head>
  <title>Reset password — Civitas</title>
</svelte:head>

<section class="mx-auto w-full max-w-md px-5 py-16 sm:px-6">
  <div class="mb-3 font-mono text-[11px] uppercase tracking-[0.2em] text-ink-400">Civitas</div>
  <h1 class="font-serif text-[34px] font-semibold leading-[1.1] tracking-[-0.01em]">
    Choose a new password
  </h1>

  <div class="mt-7 space-y-4">
    {#if errorMessage}
      <Banner tone="error" title="Reset failed">{errorMessage}</Banner>
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
        label="Reset token"
        required
        value={data.prefilledToken}
        hint={data.prefilledToken
          ? 'Token pre-filled from the link in your email.'
          : 'Paste the token from the reset email, or follow the link in it.'}
      />
      <TextField
        name="new_password"
        label="New password"
        type="password"
        required
        autocomplete="new-password"
        minlength={12}
        hint="At least 12 characters. Longer is better than complex."
      />
      <div class="flex flex-wrap items-center justify-between gap-4">
        <Button type="submit" loading={submitting}>Reset password</Button>
        <a href="/auth/forgot-password" class="text-sm text-accent-600 hover:underline">
          Need a new link?
        </a>
      </div>
    </form>
  </div>
</section>
