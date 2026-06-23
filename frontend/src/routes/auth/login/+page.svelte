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
  <title>Log in — Civitas</title>
</svelte:head>

<section class="mx-auto w-full max-w-md px-5 py-16 sm:px-6">
  <div class="mb-3 font-mono text-[11px] uppercase tracking-[0.2em] text-ink-400">Civitas</div>
  <h1 class="font-serif text-[34px] font-semibold leading-[1.1] tracking-[-0.01em]">Log in</h1>

  <div class="mt-7 space-y-4">
    {#if data.verified}
      <Banner tone="success" title="Email verified">You can now log in.</Banner>
    {/if}
    {#if data.reset}
      <Banner tone="success" title="Password changed">Log in with your new password.</Banner>
    {/if}
    {#if errorMessage}
      <Banner tone="error" title="Login failed">{errorMessage}</Banner>
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
      <TextField
        name="password"
        label="Password"
        type="password"
        required
        autocomplete="current-password"
        minlength={12}
      />
      <div class="flex flex-wrap items-center justify-between gap-4">
        <Button type="submit" loading={submitting}>Log in</Button>
        <div class="flex gap-4 text-sm">
          <a href="/auth/forgot-password" class="text-accent-600 hover:underline">
            Forgot password?
          </a>
          <a href="/auth/register" class="text-accent-600 hover:underline">
            Don't have an account?
          </a>
        </div>
      </div>
    </form>
  </div>
</section>
