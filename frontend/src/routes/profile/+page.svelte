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

<section class="mx-auto max-w-civic px-5 pb-20 pt-14 sm:px-10">
  <div class="mb-3.5 font-mono text-[11px] uppercase tracking-[0.2em] text-ink-400">Account</div>
  <h1
    class="font-serif text-[clamp(40px,5.4vw,60px)] font-semibold leading-[1.04] tracking-[-0.015em]"
  >
    Your profile
  </h1>

  {#if !user.email_verified}
    <div class="mt-6 max-w-2xl">
      <Banner tone="warning" title="Email not verified">
        You can browse and read, but voting and posting require email verification.
        <a href="/auth/verify-email" class="font-medium underline">Verify now →</a>
      </Banner>
    </div>
  {/if}

  <dl
    class="mt-8 max-w-2xl divide-y divide-line overflow-hidden rounded border border-line bg-card"
  >
    <div class="grid grid-cols-[max-content_1fr] items-center gap-x-8 px-5 py-4">
      <dt class="font-mono text-[10px] uppercase tracking-[0.12em] text-ink-400">Display name</dt>
      <dd class="text-[15px] font-medium">{user.display_name}</dd>
    </div>
    <div class="grid grid-cols-[max-content_1fr] items-center gap-x-8 px-5 py-4">
      <dt class="font-mono text-[10px] uppercase tracking-[0.12em] text-ink-400">Email</dt>
      <dd class="flex items-center gap-2 text-[15px]">
        {user.email}
        {#if user.email_verified}
          <span
            class="inline-flex rounded-full bg-affirm-50 px-2 py-0.5 font-mono text-[10px] uppercase tracking-[0.08em] text-affirm-600"
            >verified</span
          >
        {:else}
          <span
            class="inline-flex rounded-full bg-ochre-50 px-2 py-0.5 font-mono text-[10px] uppercase tracking-[0.08em] text-ochre-600"
            >pending</span
          >
        {/if}
      </dd>
    </div>
    <div class="grid grid-cols-[max-content_1fr] items-center gap-x-8 px-5 py-4">
      <dt class="font-mono text-[10px] uppercase tracking-[0.12em] text-ink-400">User ID</dt>
      <dd class="font-mono text-[12px] text-ink-600">{user.id}</dd>
    </div>
    <div class="grid grid-cols-[max-content_1fr] items-center gap-x-8 px-5 py-4">
      <dt class="font-mono text-[10px] uppercase tracking-[0.12em] text-ink-400">Member since</dt>
      <dd class="text-[15px] tabular-nums">{new Date(user.created_at).toLocaleDateString()}</dd>
    </div>
  </dl>

  <form method="POST" action="/auth/logout" class="mt-6">
    <Button type="submit" variant="secondary">Log out</Button>
  </form>
</section>
