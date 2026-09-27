<script lang="ts">
  import '../app.css';
  import type { Snippet } from 'svelte';
  import type { LayoutData } from './$types';
  import { page } from '$app/stores';
  import { currentUser } from '$lib/stores/auth';
  import { initials } from '$lib/utils/text';
  import { env } from '$env/dynamic/public';
  import { createI18n, setI18n, LOCALES, LOCALE_NAMES } from '$lib/i18n';

  let { data, children }: { data: LayoutData; children: Snippet } = $props();

  const translator = $derived(createI18n(data.locale, data.messages));
  setI18n(() => translator);
  const t = (key: string, params?: Record<string, string | number>) => translator.t(key, params);

  // Mirror the SSR-loaded user into the reactive store so components that
  // subscribe see the same value across navigations.
  $effect(() => {
    currentUser.set(data.currentUser);
  });

  const path = $derived($page.url.pathname);
  const onProposals = $derived(path === '/proposals' || path.startsWith('/proposals/'));
  const onDelegations = $derived(path.startsWith('/delegations'));
  const onOperator = $derived(path.startsWith('/operator'));

  const navLink = 'pb-0.5 transition-colors hover:text-ink-900 focus-visible:text-ink-900';
  const navActive = 'border-b border-ink-900 text-ink-900';
  const navIdle = 'text-ink-600';

  // Sandbox banner: shown only when PUBLIC_DEMO_MODE=true (demo deployments).
  // Unset/false in real production keeps it hidden.
  const demoMode = env.PUBLIC_DEMO_MODE === 'true';
</script>

<svelte:head>
  <!-- No default <title> here: every page sets its own, and a layout title
       can win over a page's in server rendering. -->
  <meta name="description" content={t('layout.meta_description')} />
</svelte:head>

<div class="flex min-h-full flex-col">
  {#if demoMode}
    <div class="bg-band text-band-ink" role="note" aria-label={t('layout.demo_label')}>
      <div class="mx-auto flex max-w-civic items-center gap-2.5 px-5 py-2 sm:px-10">
        <span class="h-1.5 w-1.5 flex-none rounded-full bg-ochre-600" aria-hidden="true"></span>
        <p class="font-mono text-[10px] uppercase leading-[1.5] tracking-[0.14em] text-band-ink">
          {t('layout.demo_notice')}
        </p>
      </div>
    </div>
  {/if}
  <header class="sticky top-0 z-50 border-b border-line bg-paper/80 backdrop-blur-[10px]">
    <!-- On narrow phones, and in languages with longer words, the nav wraps
         under the name instead of pushing the page sideways. -->
    <div
      class="mx-auto flex max-w-civic flex-wrap items-center justify-between gap-x-6 gap-y-2 px-5 py-3 sm:px-10"
    >
      <a href="/" class="flex items-center gap-3 text-ink-900" aria-label={t('layout.home_label')}>
        <span class="font-serif text-[19px] font-bold tracking-[0.02em]">Civitas</span>
        <span class="h-[5px] w-[5px] rounded-full bg-accent-600" aria-hidden="true"></span>
      </a>
      <nav
        aria-label={t('layout.nav_primary')}
        class="flex items-center gap-4 font-mono text-[11px] uppercase tracking-[0.12em] sm:gap-7"
      >
        <a
          href="/proposals"
          class="{navLink} {onProposals ? navActive : navIdle}"
          aria-current={onProposals ? 'page' : undefined}>{t('layout.nav_proposals')}</a
        >
        <a
          href="/delegations"
          class="{navLink} {onDelegations ? navActive : navIdle}"
          aria-current={onDelegations ? 'page' : undefined}>{t('layout.nav_delegations')}</a
        >
        {#if data.currentUser?.is_operator}
          <!-- No room on phones; the profile page links it there. -->
          <a
            href="/operator"
            class="hidden sm:inline {navLink} {onOperator ? navActive : navIdle}"
            aria-current={onOperator ? 'page' : undefined}>{t('layout.nav_operator')}</a
          >
        {/if}
        {#if data.currentUser}
          <a
            href="/profile"
            class="flex h-[30px] w-[30px] items-center justify-center rounded-full bg-ink-900 font-serif text-[12px] font-semibold text-white"
            aria-label={t('layout.profile_label', { name: data.currentUser.display_name })}
            title={data.currentUser.display_name}
          >
            {initials(data.currentUser.display_name)}
          </a>
        {:else}
          <a
            href="/auth/login"
            class="rounded-full bg-ink-900 px-3.5 py-2 text-white transition-opacity hover:opacity-90"
            >{t('layout.sign_in')}</a
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
          {t('layout.tagline')} ·
          <a
            href="https://www.gnu.org/licenses/agpl-3.0.html"
            class="hover:text-ink-600 hover:underline"
            rel="noopener noreferrer">AGPL-3.0</a
          >
        </span>
      </div>
      <nav
        aria-label={t('layout.nav_secondary')}
        class="flex items-center gap-4 font-mono text-[11px] uppercase tracking-[0.1em] text-ink-400"
      >
        <a href="/about" class="hover:text-ink-600 hover:underline">{t('layout.about')}</a>
        <a href="/topics" class="hover:text-ink-600 hover:underline">{t('layout.topics')}</a>
        <a
          href="https://github.com/zarathustracode/civitas"
          class="hover:text-ink-600 hover:underline"
          rel="noopener noreferrer">{t('layout.source')}</a
        >
      </nav>
      <form method="POST" action="/locale" class="flex w-full items-center gap-3 sm:w-auto">
        <input type="hidden" name="redirect" value={$page.url.pathname + $page.url.search} />
        <span
          id="language-label"
          class="font-mono text-[11px] uppercase tracking-[0.1em] text-ink-400"
          >{t('layout.language')}</span
        >
        <div role="group" aria-labelledby="language-label" class="flex gap-1.5">
          {#each LOCALES as locale (locale)}
            <button
              type="submit"
              name="locale"
              value={locale}
              lang={locale}
              aria-current={locale === data.locale ? 'true' : undefined}
              class="rounded-full border px-2.5 py-1 text-[12px] transition-colors {locale ===
              data.locale
                ? 'border-ink-900 bg-ink-900 text-white'
                : 'border-line text-ink-600 hover:border-ink-400'}">{LOCALE_NAMES[locale]}</button
            >
          {/each}
        </div>
      </form>
    </div>
  </footer>
</div>
