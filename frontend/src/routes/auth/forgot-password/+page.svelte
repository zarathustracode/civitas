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

<section class="prose-civic">
  <h1>Forgot your password?</h1>
  <p>Enter your account email and we'll send a reset link. The link is valid for one hour.</p>

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
    <div class="flex items-center justify-between">
      <Button type="submit" loading={submitting}>Send reset link</Button>
      <a href="/auth/login" class="text-sm text-accent-600 hover:underline">Back to login</a>
    </div>
  </form>
</section>
