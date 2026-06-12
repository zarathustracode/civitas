<script lang="ts">
  import type { Proposal, Tally, VoteChoice } from '$lib/types/domain';

  let { proposal, tally }: { proposal: Proposal; tally: Tally } = $props();

  type Outcome =
    | { kind: 'verdict'; leader: VoteChoice; leaderWeight: number; counted: number; pct: number }
    | { kind: 'no_quorum' }
    | { kind: 'tie'; leaders: VoteChoice[] };

  const choiceLabel: Record<VoteChoice, string> = {
    yes: 'Yes',
    no: 'No',
    abstain: 'Abstain'
  };

  const outcome = $derived.by<Outcome>(() => {
    const yes = parseFloat(tally.yes);
    const no = parseFloat(tally.no);
    const abstain = parseFloat(tally.abstain);
    const counted = yes + no + abstain;
    if (counted <= 0) return { kind: 'no_quorum' };

    const entries: [VoteChoice, number][] = [
      ['yes', yes],
      ['no', no],
      ['abstain', abstain]
    ];
    const max = Math.max(yes, no, abstain);
    const leaders = entries.filter(([, w]) => w === max).map(([c]) => c);
    if (leaders.length !== 1) return { kind: 'tie', leaders };
    return {
      kind: 'verdict',
      leader: leaders[0] as VoteChoice,
      leaderWeight: max,
      counted,
      pct: (max / counted) * 100
    };
  });

  const closedAt = $derived(
    proposal.voting_ends_at
      ? new Date(proposal.voting_ends_at).toLocaleString(undefined, {
          dateStyle: 'medium',
          timeStyle: 'short'
        })
      : null
  );

  const tone = $derived(outcome.kind === 'verdict' && outcome.leader === 'yes' ? 'win' : 'neutral');
</script>

<section
  aria-labelledby="results-heading"
  class="rounded-lg border p-4 {tone === 'win'
    ? 'border-affirm-500 bg-affirm-50'
    : 'border-ink-200 bg-ink-50'}"
>
  <p class="text-xs uppercase tracking-wide text-ink-600">Final result</p>
  <h2 id="results-heading" class="mt-1 text-2xl font-semibold">
    {#if outcome.kind === 'no_quorum'}
      No verdict — no votes were counted
    {:else if outcome.kind === 'tie'}
      Tie — {outcome.leaders.map((c) => choiceLabel[c]).join(' / ')}
    {:else}
      {choiceLabel[outcome.leader]} ({outcome.pct.toFixed(0)}%)
    {/if}
  </h2>
  <p class="text-ink-700 mt-1 text-sm">
    {tally.counted_voters} of {tally.eligible_voters} eligible voters counted{#if closedAt}
      · closed {closedAt}{/if}.
  </p>
</section>
