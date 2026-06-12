<script lang="ts">
  import { enhance } from '$app/forms';
  import Button from '$lib/components/Button.svelte';
  import TextField from '$lib/components/TextField.svelte';
  import Banner from '$lib/components/Banner.svelte';
  import { friendlyMessage, ApiError } from '$lib/api/errors';
  import type { ActionData, PageData } from './$types';

  let { data, form }: { data: PageData; form: ActionData } = $props();
  let submitting = $state(false);
  let resending = $state(false);

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
      We sent a verification link to your email. Follow the link or paste the token below.
    </Banner>
  {/if}
  {#if form?.resent}
    <Banner tone="success" title="Verification email sent">
      If an unverified account exists for that address, a fresh link is on its way.
    </Banner>
  {/if}
  {#if errorMessage}
    <Banner tone="error" title="Verification failed">{errorMessage}</Banner>
  {/if}

  <form
    method="POST"
    action="?/verify"
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
        ? 'Token pre-filled from your verification link. Submit to verify.'
        : 'The token arrives by email. Follow the link in it, or paste the token here.'}
    />
    <Button type="submit" loading={submitting}>Verify email</Button>
  </form>

  <h2>Didn't get the email?</h2>
  <p>Enter your account email and we'll send a fresh verification link.</p>

  <form
    method="POST"
    action="?/resend"
    class="flex flex-col gap-4"
    use:enhance={() => {
      resending = true;
      return async ({ update }) => {
        await update();
        resending = false;
      };
    }}
  >
    <TextField name="email" label="Email" type="email" required autocomplete="email" />
    <Button type="submit" variant="secondary" loading={resending}>Resend verification email</Button>
  </form>
</section>
