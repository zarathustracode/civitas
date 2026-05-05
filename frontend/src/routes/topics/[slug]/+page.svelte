<script lang="ts">
  import type { PageData } from './$types';
  import ProposalCard from '$lib/components/ProposalCard.svelte';
  let { data }: { data: PageData } = $props();
</script>

<svelte:head>
  <title>{data.topic.name} — Civitas</title>
</svelte:head>

<section class="prose-civic">
  <p class="text-sm text-ink-600">
    <a href="/topics" class="hover:underline">Topics</a> ›
  </p>
  <h1>{data.topic.name}</h1>
  {#if data.topic.description}
    <p>{data.topic.description}</p>
  {/if}

  <h2>Proposals</h2>
  {#if data.proposals.length === 0}
    <p class="text-ink-600">No proposals on this topic yet.</p>
  {:else}
    <ul class="space-y-3">
      {#each data.proposals as proposal (proposal.id)}
        <li><ProposalCard {proposal} /></li>
      {/each}
    </ul>
  {/if}
</section>
