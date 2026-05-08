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
  <title>Verify your email — Civitas</title>
</svelte:head>

<section class="prose-civic">
  <h1>Verify your email</h1>

  {#if data.registered}
    <Banner tone="success" title="Account created">
      We sent a verification link to your email. Paste the token below or follow the link.
    </Banner>
  {/if}
  {#if errorMessage}
    <Banner tone="error" title="Verification failed">{errorMessage}</Banner>
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
      label="Verification token"
      required
      value={data.prefilledToken}
      hint={data.prefilledToken
        ? 'Token pre-filled from the registration response (dev mode). Submit to verify.'
        : 'In production the token arrives by email. In dev with DEV_RETURN_VERIFICATION_TOKEN=true the registration redirect pre-fills it for you.'}
    />
    <Button type="submit" loading={submitting}>Verify email</Button>
  </form>
</section>
