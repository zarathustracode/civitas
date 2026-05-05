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

<section class="prose-civic">
  <h1>Create an account</h1>
  <p>
    Email and a display name. Choose a strong password (12+ characters). After registration we send
    a verification link to your email — you'll need to confirm before you can vote.
  </p>

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
    <div class="flex items-center justify-between">
      <Button type="submit" loading={submitting}>Create account</Button>
      <a href="/auth/login" class="text-sm text-accent-600 hover:underline">
        Already have an account?
      </a>
    </div>
  </form>
</section>
