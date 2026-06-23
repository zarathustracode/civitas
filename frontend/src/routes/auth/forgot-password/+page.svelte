<script lang="ts">
  import { enhance } from '$app/forms';
  import Button from '$lib/components/Button.svelte';
  import TextField from '$lib/components/TextField.svelte';
  import Banner from '$lib/components/Banner.svelte';
  import { friendlyMessage, ApiError } from '$lib/api/errors';
  import type { ActionData } from './$types';

  let { form }: { form: ActionData } = $props();
  let submitting = $state(false);

  const errorMessage = $derived(
    form?.code ? friendlyMessage(new ApiError(form.code, form.code, 0)) : null
  );
</script>

<svelte:head>
  <title>Forgot password — Civitas</title>
</svelte:head>

<section class="mx-auto w-full max-w-md px-5 py-16 sm:px-6">
  <div class="mb-3 font-mono text-[11px] uppercase tracking-[0.2em] text-ink-400">Civitas</div>
  <h1 class="font-serif text-[34px] font-semibold leading-[1.1] tracking-[-0.01em]">
    Forgot your password?
  </h1>
  <p class="mt-3 font-serif text-[17px] leading-[1.5] text-ink-600">
    Enter your account email and we'll send a reset link. The link is valid for one hour.
  </p>

  <div class="mt-7 space-y-4">
    {#if form?.sent}
      <Banner tone="success" title="Check your email">
        If an account exists for that address, a reset link is on its way.
      </Banner>
    {/if}
    {#if errorMessage}
      <Banner tone="error" title="Request failed">{errorMessage}</Banner>
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
        label="Email"
        type="email"
        required
        autocomplete="email"
        value={form?.email ?? ''}
      />
      <div class="flex flex-wrap items-center justify-between gap-4">
        <Button type="submit" loading={submitting}>Send reset link</Button>
        <a href="/auth/login" class="text-sm text-accent-600 hover:underline">Back to login</a>
      </div>
    </form>
  </div>
</section>
