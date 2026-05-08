<script lang="ts">
  import type { PageData } from './$types';
  import ProposalCard from '$lib/components/ProposalCard.svelte';
  import TopicStatsPanel from '$lib/components/TopicStatsPanel.svelte';
  let { data }: { data: PageData } = $props();
</script>

<svelte:head>
  <title>{data.topic.name} — Civitas</title>
</svelte:head>

<section class="space-y-6">
  <header class="prose-civic">
    <p class="text-sm text-ink-600">
      <a href="/topics" class="hover:underline">Topics</a> ›
    </p>
    <h1>{data.topic.name}</h1>
    {#if data.topic.description}
      <p>{data.topic.description}</p>
    {/if}
  </header>

  <TopicStatsPanel stats={data.stats} />

  <div class="space-y-3">
    <h2 class="text-lg font-semibold">Proposals</h2>
    {#if data.proposals.length === 0}
      <p class="rounded-md bg-ink-100 px-4 py-3 text-sm text-ink-800">
        No proposals on this topic yet.
      </p>
    {:else}
      <ul class="space-y-3">
        {#each data.proposals as proposal (proposal.id)}
          <li><ProposalCard {proposal} /></li>
        {/each}
      </ul>
    {/if}
  </div>
</section>
