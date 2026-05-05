<script lang="ts">
  import '../app.css';
  import type { Snippet } from 'svelte';
  import type { LayoutData } from './$types';
  import { currentUser } from '$lib/stores/auth';

  let { data, children }: { data: LayoutData; children: Snippet } = $props();

  // Mirror the SSR-loaded user into the reactive store so components that
  // subscribe see the same value across navigations.
  $effect(() => {
    currentUser.set(data.currentUser);
  });
</script>

<div class="flex min-h-full flex-col">
  <header class="border-b border-ink-200 bg-white">
    <nav aria-label="Primary" class="mx-auto flex max-w-3xl items-center justify-between gap-4 p-4">
      <a href="/" class="text-xl font-semibold tracking-tight">Civitas</a>
      <ul class="flex items-center gap-2 text-sm sm:gap-4">
        <li><a href="/proposals" class="hover:underline">Proposals</a></li>
        <li><a href="/topics" class="hover:underline">Topics</a></li>
        {#if data.currentUser}
          <li><a href="/delegations" class="hover:underline">Delegations</a></li>
          <li>
            <a href="/profile" class="hover:underline">{data.currentUser.display_name}</a>
          </li>
        {:else}
          <li><a href="/auth/login" class="hover:underline">Log in</a></li>
          <li>
            <a
              href="/auth/register"
              class="rounded-md bg-accent-600 px-3 py-1.5 text-white hover:bg-accent-700"
            >
              Register
            </a>
          </li>
        {/if}
      </ul>
    </nav>
  </header>

  <main id="main" class="mx-auto w-full max-w-3xl flex-1 p-4 sm:p-6">
    {@render children()}
  </main>

  <footer class="border-t border-ink-200 bg-white text-sm text-ink-600">
    <div class="mx-auto flex max-w-3xl flex-wrap items-center justify-between gap-2 p-4">
      <p>
        Civitas — open source under
        <a href="https://www.gnu.org/licenses/agpl-3.0.html" class="underline">AGPL-3.0</a>.
      </p>
      <ul class="flex gap-3">
        <li><a href="/about" class="hover:underline">About</a></li>
        <li>
          <a
            href="https://github.com/zarathustracode/civitas"
            class="hover:underline"
            rel="noopener noreferrer"
          >
            Source
          </a>
        </li>
      </ul>
    </div>
  </footer>
</div>
