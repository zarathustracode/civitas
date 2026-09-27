<script lang="ts">
  import { getI18n } from '$lib/i18n';
  import Rich from '$lib/components/Rich.svelte';

  const i18n = getI18n();

  const REPO = 'https://github.com/zarathustracode/civitas';
</script>

<svelte:head>
  <title>{i18n.t('about.page_title')} — Civitas</title>
</svelte:head>

<section class="mx-auto max-w-civic px-5 pb-20 pt-14 sm:px-10">
  <div class="mb-3.5 font-mono text-[11px] uppercase tracking-[0.2em] text-ink-400">
    {i18n.t('about.eyebrow')}
  </div>
  <h1
    class="font-serif text-[clamp(40px,5.4vw,60px)] font-semibold leading-[1.04] tracking-[-0.015em]"
  >
    {i18n.t('about.title')}
  </h1>

  <div class="prose-civic mt-8 text-[17px]">
    <p>
      <Rich parts={i18n.rich('about.intro')} href="{REPO}/blob/main/docs/roadmap.md" />
    </p>

    <h2>{i18n.t('about.governance_title')}</h2>
    <p>{i18n.t('about.governance_body')}</p>
    <p>
      <Rich parts={i18n.rich('about.governance_plan')} href="{REPO}/blob/main/GOVERNANCE.md" />
    </p>

    <h2>{i18n.t('about.source_title')}</h2>
    <p>
      <Rich
        parts={i18n.rich('about.source_license')}
        href="https://www.gnu.org/licenses/agpl-3.0.html"
      />
    </p>
    <p>
      <Rich parts={i18n.rich('about.source_repo')} href={REPO} rel="noopener noreferrer" />
    </p>

    <h2>{i18n.t('about.issues_title')}</h2>
    <p>
      <Rich parts={i18n.rich('about.issues_body')} href="{REPO}/blob/main/SECURITY.md" />
    </p>
  </div>
</section>
