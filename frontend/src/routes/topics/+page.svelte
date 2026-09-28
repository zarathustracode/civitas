<script lang="ts">
  import type { PageData } from './$types';
  import { getI18n } from '$lib/i18n';
  let { data }: { data: PageData } = $props();
  const i18n = getI18n();
</script>

<svelte:head>
  <title>{i18n.t('topics.title')} — Civitas</title>
</svelte:head>

<section class="mx-auto max-w-civic px-5 pb-20 pt-14 sm:px-10">
  <div class="mb-3.5 font-mono text-[11px] uppercase tracking-[0.2em] text-ink-400">
    {i18n.t('topics.eyebrow')}
  </div>
  <h1
    class="font-serif text-[clamp(40px,5.4vw,60px)] font-semibold leading-[1.04] tracking-[-0.015em]"
  >
    {i18n.t('topics.title')}
  </h1>
  <p class="mt-[18px] max-w-[58ch] font-serif text-[20px] leading-[1.5] text-ink-600">
    {i18n.t('topics.intro')}
  </p>

  {#if data.topics.length === 0}
    <p
      class="mt-8 rounded border border-dashed border-line px-6 py-6 font-serif text-[18px] text-ink-600"
    >
      {i18n.t('topics.empty')}{#if data.currentUser}
        {' '}{i18n.t('topics.empty_hint')}{/if}
    </p>
  {:else}
    <div class="mt-8 grid gap-3 sm:grid-cols-2">
      {#each data.topics as topic (topic.id)}
        <a
          href="/topics/{topic.slug}"
          class="block rounded border border-line bg-card p-5 transition-colors hover:border-ink-400"
        >
          <p class="font-serif text-[19px] font-semibold text-ink-900">{topic.name}</p>
          {#if topic.description}
            <p class="mt-1.5 font-serif text-[15px] leading-[1.45] text-ink-600">
              {topic.description}
            </p>
          {/if}
        </a>
      {/each}
    </div>
  {/if}
</section>
