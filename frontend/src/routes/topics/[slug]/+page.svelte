<script lang="ts">
  import type { PageData } from './$types';
  import ProposalCard from '$lib/components/ProposalCard.svelte';
  import TopicStatsPanel from '$lib/components/TopicStatsPanel.svelte';
  let { data }: { data: PageData } = $props();
</script>

<svelte:head>
  <title>{data.topic.name} — Civitas</title>
</svelte:head>

<section class="mx-auto max-w-civic px-5 pb-20 pt-14 sm:px-10">
  <p class="mb-6 font-mono text-[11px] uppercase tracking-[0.16em] text-ink-400">
    <a href="/topics" class="hover:text-ink-600 hover:underline">Topics</a> ›
  </p>
  <h1
    class="font-serif text-[clamp(40px,5.4vw,60px)] font-semibold leading-[1.04] tracking-[-0.015em]"
  >
    {data.topic.name}
  </h1>
  {#if data.topic.description}
    <p class="mt-[18px] max-w-[60ch] font-serif text-[20px] leading-[1.5] text-ink-600">
      {data.topic.description}
    </p>
  {/if}

  <div class="mt-9">
    <TopicStatsPanel stats={data.stats} />
  </div>

  <div class="mt-12">
    <h2 class="mb-5 font-serif text-[24px] font-semibold tracking-[-0.01em]">Proposals</h2>
    {#if data.proposals.length === 0}
      <p
        class="rounded border border-dashed border-line px-6 py-6 font-serif text-[18px] text-ink-600"
      >
        No proposals on this topic yet.
      </p>
    {:else}
      <div class="grid gap-3">
        {#each data.proposals as proposal (proposal.id)}
          <ProposalCard {proposal} />
        {/each}
      </div>
    {/if}
  </div>
</section>
