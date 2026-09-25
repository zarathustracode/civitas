<script lang="ts">
  import { enhance } from '$app/forms';
  import Button from '$lib/components/Button.svelte';
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
  <title>Finish signing in — Civitas</title>
</svelte:head>

<section class="mx-auto w-full max-w-md px-5 py-16 sm:px-6">
  <div class="mb-3 font-mono text-[11px] uppercase tracking-[0.2em] text-ink-400">Civitas</div>
  <h1 class="font-serif text-[34px] font-semibold leading-[1.1] tracking-[-0.01em]">
    Finish signing in
  </h1>

  <div class="mt-7 space-y-4">
    {#if !data.token}
      <Banner tone="error" title="No sign-in link">
        This page needs the link from your email.
        <a href="/auth/login-link" class="font-medium underline">Request a new link →</a>
      </Banner>
    {:else}
      {#if errorMessage}
        <Banner tone="error" title="Could not sign you in">
          {errorMessage} Links work once and expire after 15 minutes.
          <a href="/auth/login-link" class="font-medium underline">Request a new link →</a>
        </Banner>
      {:else}
        <p class="font-serif text-[17px] leading-[1.5] text-ink-600">
          Press the button to sign in on this device. The link then stops working.
        </p>
      {/if}

      <form
        method="POST"
        use:enhance={() => {
          submitting = true;
          return async ({ update }) => {
            await update();
            submitting = false;
          };
        }}
      >
        <input type="hidden" name="token" value={data.token} />
        <Button type="submit" loading={submitting}>Sign in</Button>
      </form>
    {/if}
  </div>
</section>
