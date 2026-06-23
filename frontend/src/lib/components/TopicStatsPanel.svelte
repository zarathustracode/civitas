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
  class="grid gap-px overflow-hidden rounded border border-line bg-line sm:grid-cols-3"
>
  <h2 id="topic-stats-heading" class="sr-only">Topic activity</h2>

  <div class="bg-card px-6 py-5">
    <div class="font-mono text-[10px] uppercase tracking-[0.12em] text-ink-400">Proposals</div>
    <div class="mt-1.5 font-mono text-[28px] font-medium tabular-nums">{totalProposals}</div>
    {#if totalProposals > 0}
      <p class="mt-1.5 font-mono text-[11px] leading-[1.5] text-ink-400">
        {stats.proposal_counts.voting} voting · {stats.proposal_counts.deliberation} in deliberation
        · {stats.proposal_counts.closed} closed
      </p>
    {/if}
  </div>

  <div class="bg-card px-6 py-5">
    <div class="font-mono text-[10px] uppercase tracking-[0.12em] text-ink-400">
      Active delegations
    </div>
    <div class="mt-1.5 font-mono text-[28px] font-medium tabular-nums">
      {stats.active_delegations}
    </div>
    <p class="mt-1.5 font-mono text-[11px] leading-[1.5] text-ink-400">
      votes routed via delegation on this topic
    </p>
  </div>

  <div class="bg-card px-6 py-5">
    <div class="font-mono text-[10px] uppercase tracking-[0.12em] text-ink-400">Top delegates</div>
    {#if stats.top_delegates.length === 0}
      <p class="mt-1.5 font-serif text-[14px] text-ink-600">Nobody is delegating here yet.</p>
    {:else}
      <ol class="mt-2 space-y-1">
        {#each stats.top_delegates as d (d.id)}
          <li class="flex items-baseline justify-between gap-2">
            <span class="font-serif text-[15px]">{d.display_name}</span>
            <span class="font-mono text-[12px] tabular-nums text-ink-400">{d.incoming}</span>
          </li>
        {/each}
      </ol>
    {/if}
  </div>
</section>
