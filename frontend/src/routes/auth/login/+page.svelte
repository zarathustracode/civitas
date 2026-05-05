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
  <title>Log in — Civitas</title>
</svelte:head>

<section class="prose-civic">
  <h1>Log in</h1>

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
    <div class="flex items-center justify-between">
      <Button type="submit" loading={submitting}>Log in</Button>
      <a href="/auth/register" class="text-sm text-accent-600 hover:underline">
        Don't have an account?
      </a>
    </div>
  </form>
</section>
