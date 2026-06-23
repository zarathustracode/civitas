<script lang="ts">
  import '../app.css';
  import type { Snippet } from 'svelte';
  import type { LayoutData } from './$types';
  import { page } from '$app/stores';
  import { currentUser } from '$lib/stores/auth';
  import { initials } from '$lib/utils/text';
  import { env } from '$env/dynamic/public';

  let { data, children }: { data: LayoutData; children: Snippet } = $props();

  // Mirror the SSR-loaded user into the reactive store so components that
  // subscribe see the same value across navigations.
  $effect(() => {
    currentUser.set(data.currentUser);
  });

  const path = $derived($page.url.pathname);
  const onProposals = $derived(path === '/proposals' || path.startsWith('/proposals/'));
  const onDelegations = $derived(path.startsWith('/delegations'));

  const navLink = 'pb-0.5 transition-colors hover:text-ink-900 focus-visible:text-ink-900';
  const navActive = 'border-b border-ink-900 text-ink-900';
  const navIdle = 'text-ink-600';

  // Sandbox banner: shown only when PUBLIC_DEMO_MODE=true (demo deployments).
  // Unset/false in real production keeps it hidden.
  const demoMode = env.PUBLIC_DEMO_MODE === 'true';
</script>

<div class="flex min-h-full flex-col">
  {#if demoMode}
    <div class="bg-band text-band-ink" role="note" aria-label="Sandbox demo notice">
      <div class="mx-auto flex max-w-civic items-center gap-2.5 px-5 py-2 sm:px-10">
        <span class="h-1.5 w-1.5 flex-none rounded-full bg-ochre-600" aria-hidden="true"></span>
        <p class="font-mono text-[10px] uppercase leading-[1.5] tracking-[0.14em] text-band-ink">
          Sandbox demo · data is seeded for demonstration — votes, delegations, and accounts here
          are not binding.
        </p>
      </div>
    </div>
  {/if}
  <header class="sticky top-0 z-50 border-b border-line bg-paper/80 backdrop-blur-[10px]">
    <div class="mx-auto flex max-w-civic items-center justify-between gap-6 px-5 py-3 sm:px-10">
      <a href="/" class="flex items-center gap-3 text-ink-900" aria-label="Civitas — home">
        <span class="font-serif text-[19px] font-bold tracking-[0.02em]">Civitas</span>
        <span class="h-[5px] w-[5px] rounded-full bg-accent-600" aria-hidden="true"></span>
      </a>
      <nav
        aria-label="Primary"
        class="flex items-center gap-5 font-mono text-[11px] uppercase tracking-[0.12em] sm:gap-7"
      >
        <a
          href="/proposals"
          class="{navLink} {onProposals ? navActive : navIdle}"
          aria-current={onProposals ? 'page' : undefined}>Proposals</a
        >
        <a
          href="/delegations"
          class="{navLink} {onDelegations ? navActive : navIdle}"
          aria-current={onDelegations ? 'page' : undefined}>Delegations</a
        >
        {#if data.currentUser}
          <a
            href="/profile"
            class="flex h-[30px] w-[30px] items-center justify-center rounded-full bg-ink-900 font-serif text-[12px] font-semibold text-white"
            aria-label="Your profile — {data.currentUser.display_name}"
            title={data.currentUser.display_name}
          >
            {initials(data.currentUser.display_name)}
          </a>
        {:else}
          <a
            href="/auth/login"
            class="rounded-full bg-ink-900 px-3.5 py-2 text-white transition-opacity hover:opacity-90"
            >Sign in</a
          >
        {/if}
      </nav>
    </div>
  </header>

  <main id="main" class="flex-1">
    {@render children()}
  </main>

  <footer class="border-t border-line">
    <div
      class="mx-auto flex max-w-civic flex-wrap items-center justify-between gap-4 px-5 py-9 sm:px-10"
    >
      <div class="flex items-center gap-3">
        <span class="font-serif text-base font-bold text-ink-900">Civitas</span>
        <span class="font-mono text-[11px] tracking-[0.06em] text-ink-400">
          open civic infrastructure ·
          <a
            href="https://www.gnu.org/licenses/agpl-3.0.html"
            class="hover:text-ink-600 hover:underline"
            rel="noopener noreferrer">AGPL-3.0</a
          >
        </span>
      </div>
      <nav
        aria-label="Secondary"
        class="flex items-center gap-4 font-mono text-[11px] uppercase tracking-[0.1em] text-ink-400"
      >
        <a href="/about" class="hover:text-ink-600 hover:underline">About</a>
        <a href="/topics" class="hover:text-ink-600 hover:underline">Topics</a>
        <a
          href="https://github.com/zarathustracode/civitas"
          class="hover:text-ink-600 hover:underline"
          rel="noopener noreferrer">Source</a
        >
      </nav>
    </div>
  </footer>
</div>
