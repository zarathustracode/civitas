<script lang="ts">
  import type { PageData } from './$types';
  import Button from '$lib/components/Button.svelte';
  import Banner from '$lib/components/Banner.svelte';

  let { data }: { data: PageData } = $props();
  // Layout guarantees currentUser is set on protected routes.
  const user = $derived(data.currentUser!);
</script>

<svelte:head>
  <title>Profile — Civitas</title>
</svelte:head>

<section class="space-y-6">
  <header class="prose-civic">
    <h1>Your profile</h1>
  </header>

  {#if !user.email_verified}
    <Banner tone="warning" title="Email not verified">
      You can browse and read, but voting and posting require email verification.
      <a href="/auth/verify-email" class="font-medium underline">Verify now →</a>
    </Banner>
  {/if}

  <dl class="grid grid-cols-[max-content_1fr] gap-x-4 gap-y-2 text-sm">
    <dt class="font-medium text-ink-600">Display name</dt>
    <dd>{user.display_name}</dd>

    <dt class="font-medium text-ink-600">Email</dt>
    <dd>
      {user.email}
      {#if user.email_verified}
        <span
          class="ml-1 inline-flex rounded-full bg-affirm-600/10 px-2 py-0.5 text-xs text-affirm-600"
          >verified</span
        >
      {:else}
        <span
          class="ml-1 inline-flex rounded-full bg-yellow-100 px-2 py-0.5 text-xs text-yellow-800"
          >pending</span
        >
      {/if}
    </dd>

    <dt class="font-medium text-ink-600">User ID</dt>
    <dd class="font-mono text-xs">{user.id}</dd>

    <dt class="font-medium text-ink-600">Member since</dt>
    <dd>{new Date(user.created_at).toLocaleDateString()}</dd>
  </dl>

  <form method="POST" action="/auth/logout">
    <Button type="submit" variant="secondary">Log out</Button>
  </form>
</section>
