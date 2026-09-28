<script lang="ts">
  import { page } from '$app/stores';
  import { getI18n } from '$lib/i18n';

  const i18n = getI18n();

  const title = $derived(
    $page.status === 404
      ? i18n.t('errors.page.not_found_title')
      : $page.status === 403
        ? i18n.t('errors.page.forbidden_title')
        : i18n.t('errors.page.error_title')
  );
  // Our own error() calls carry a translated message; SvelteKit's generic
  // ones ("Not Found", "Internal Error") do not, so use ours for those.
  const body = $derived(
    $page.status === 404 && $page.error?.message === 'Not Found'
      ? i18n.t('errors.page.not_found_body')
      : $page.status >= 500
        ? i18n.t('errors.page.error_body')
        : ($page.error?.message ?? i18n.t('errors.page.error_body'))
  );
</script>

<svelte:head>
  <title>{title} · Civitas</title>
</svelte:head>

<section class="mx-auto max-w-civic px-5 py-20 sm:px-10">
  <p class="mb-4 font-mono text-[11px] uppercase tracking-[0.2em] text-ink-400">{$page.status}</p>
  <h1 class="font-serif text-[clamp(32px,4.4vw,48px)] font-semibold leading-[1.1]">{title}</h1>
  <p class="mt-5 max-w-[58ch] font-serif text-[19px] leading-[1.55] text-ink-600">{body}</p>
  <p class="mt-8">
    <a href="/" class="text-accent-600 underline underline-offset-2 hover:text-accent-700"
      >{i18n.t('errors.page.home')}</a
    >
  </p>
</section>
