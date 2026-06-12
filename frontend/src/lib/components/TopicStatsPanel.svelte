<script lang="ts">
  import type { TopicStats } from '$lib/types/domain';

  let { stats }: { stats: TopicStats } = $props();

  const totalProposals = $derived(
    stats.proposal_counts.draft +
      stats.proposal_counts.deliberation +
      stats.proposal_counts.voting +
      stats.proposal_counts.closed
  );
</script>

<section
  aria-labelledby="topic-stats-heading"
  class="grid gap-4 rounded-lg border border-ink-200 bg-white p-4 sm:grid-cols-3"
>
  <h2 id="topic-stats-heading" class="sr-only">Topic activity</h2>

  <div>
    <p class="text-xs uppercase tracking-wide text-ink-600">Proposals</p>
    <p class="text-2xl font-semibold tabular-nums">{totalProposals}</p>
    {#if totalProposals > 0}
      <p class="mt-1 text-xs text-ink-600">
        {stats.proposal_counts.voting} voting · {stats.proposal_counts.deliberation} in deliberation
        · {stats.proposal_counts.closed} closed
      </p>
    {/if}
  </div>

  <div>
    <p class="text-xs uppercase tracking-wide text-ink-600">Active delegations</p>
    <p class="text-2xl font-semibold tabular-nums">{stats.active_delegations}</p>
    <p class="mt-1 text-xs text-ink-600">votes routed via delegation on this topic</p>
  </div>

  <div>
    <p class="text-xs uppercase tracking-wide text-ink-600">Top delegates</p>
    {#if stats.top_delegates.length === 0}
      <p class="mt-1 text-sm text-ink-600">Nobody is delegating on this topic yet.</p>
    {:else}
      <ol class="mt-1 space-y-0.5 text-sm">
        {#each stats.top_delegates as d (d.id)}
          <li class="flex items-baseline justify-between gap-2">
            <span>{d.display_name}</span>
            <span class="text-xs tabular-nums text-ink-600">{d.incoming}</span>
          </li>
        {/each}
      </ol>
    {/if}
  </div>
</section>
