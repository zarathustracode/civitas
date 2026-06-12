<script lang="ts">
  import type { Vote, VoteChoice } from '$lib/types/domain';

  let { votes }: { votes: Vote[] | null } = $props();

  const choiceLabel: Record<VoteChoice, string> = {
    yes: 'Yes',
    no: 'No',
    abstain: 'Abstain'
  };

  // Votes are append-only; the API returns them newest-first. Index 0 is the
  // active vote — earlier rows are superseded.
  const superseded = $derived(votes && votes.length > 1 ? votes.slice(1) : []);

  const fmt = (iso: string) =>
    new Date(iso).toLocaleString(undefined, {
      dateStyle: 'medium',
      timeStyle: 'short'
    });
</script>

{#if superseded.length > 0}
  <section
    aria-labelledby="vote-history-heading"
    class="space-y-2 rounded-md border border-ink-200 bg-white p-3 text-sm"
  >
    <h3 id="vote-history-heading" class="font-semibold">Your previous votes on this proposal</h3>
    <p class="text-ink-600">
      Votes are append-only — these were superseded by your most recent vote.
    </p>
    <ol class="space-y-1">
      {#each superseded as v (v.id)}
        <li class="flex items-baseline justify-between gap-3">
          <span><strong>{choiceLabel[v.choice]}</strong></span>
          <span class="text-xs tabular-nums text-ink-600">{fmt(v.cast_at)}</span>
        </li>
      {/each}
    </ol>
  </section>
{/if}
