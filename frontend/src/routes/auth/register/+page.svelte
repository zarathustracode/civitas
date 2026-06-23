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
  <title>Register — Civitas</title>
</svelte:head>

<section class="mx-auto w-full max-w-md px-5 py-16 sm:px-6">
  <div class="mb-3 font-mono text-[11px] uppercase tracking-[0.2em] text-ink-400">Civitas</div>
  <h1 class="font-serif text-[34px] font-semibold leading-[1.1] tracking-[-0.01em]">
    Create an account
  </h1>
  <p class="mt-3 font-serif text-[17px] leading-[1.5] text-ink-600">
    Email and a display name. Choose a strong password (12+ characters). After registration we send
    a verification link to your email — you'll need to confirm before you can vote.
  </p>

  <div class="mt-7 space-y-4">
    {#if errorMessage}
      <Banner tone="error" title="Could not register">{errorMessage}</Banner>
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
        name="display_name"
        label="Display name"
        required
        autocomplete="name"
        value={form?.display_name ?? ''}
        hint="How you appear to others. Can be a pseudonym."
      />
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
        autocomplete="new-password"
        minlength={12}
        hint="At least 12 characters. Longer is better than complex."
      />
      <div class="flex flex-wrap items-center justify-between gap-4">
        <Button type="submit" loading={submitting}>Create account</Button>
        <a href="/auth/login" class="text-sm text-accent-600 hover:underline">
          Already have an account?
        </a>
      </div>
    </form>
  </div>
</section>
