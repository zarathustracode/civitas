<script lang="ts">
  import type { PageData } from './$types';
  import ProposalCard from '$lib/components/ProposalCard.svelte';
  import type { ProposalStatus } from '$lib/types/domain';

  let { data }: { data: PageData } = $props();

  const filters: { value: ProposalStatus; label: string }[] = [
    { value: 'voting', label: 'Voting' },
    { value: 'deliberation', label: 'Deliberation' },
    { value: 'closed', label: 'Closed' },
    { value: 'draft', label: 'Draft' }
  ];
</script>

<svelte:head>
  <title>Proposals — Civitas</title>
</svelte:head>

<section class="space-y-4">
  <header class="prose-civic">
    <h1>Proposals</h1>
  </header>

  <nav aria-label="Filter by status" class="flex flex-wrap gap-2">
    {#each filters as f (f.value)}
      <a
        href="?status={f.value}"
        class="rounded-md border px-3 py-1 text-sm {data.activeStatus === f.value
          ? 'border-accent-600 bg-accent-50 text-accent-700'
          : 'border-ink-200 bg-white text-ink-800 hover:bg-ink-50'}"
        aria-current={data.activeStatus === f.value ? 'page' : undefined}
      >
        {f.label}
      </a>
    {/each}
  </nav>

  {#if data.proposals.length === 0}
    <p class="rounded-md bg-ink-100 px-4 py-3 text-sm text-ink-800">
      No proposals match this filter.
    </p>
  {:else}
    <ul class="space-y-3">
      {#each data.proposals as proposal (proposal.id)}
        <li><ProposalCard {proposal} /></li>
      {/each}
    </ul>
  {/if}
</section>
