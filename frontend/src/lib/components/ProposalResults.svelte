<script lang="ts">
  import type { Proposal, Tally, VoteChoice } from '$lib/types/domain';
  import { getI18n } from '$lib/i18n';

  let { proposal, tally }: { proposal: Proposal; tally: Tally } = $props();

  const i18n = getI18n();

  type Outcome =
    | { kind: 'verdict'; leader: VoteChoice; leaderWeight: number; counted: number; pct: number }
    | { kind: 'no_quorum' }
    | { kind: 'tie'; leaders: VoteChoice[] };

  const choiceLabel = (c: VoteChoice) => i18n.t(`common.choice.${c}`);

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
      ? i18n.date(proposal.voting_ends_at, { dateStyle: 'medium', timeStyle: 'short' })
      : null
  );

  const counts = $derived({ counted: tally.counted_voters, eligible: tally.eligible_voters });

  const tone = $derived(outcome.kind === 'verdict' && outcome.leader === 'yes' ? 'win' : 'neutral');
</script>

<section
  aria-labelledby="results-heading"
  class="rounded-lg border p-4 {tone === 'win'
    ? 'border-affirm-500 bg-affirm-50'
    : 'border-ink-200 bg-ink-50'}"
>
  <p class="text-xs uppercase tracking-wide text-ink-600">{i18n.t('tally.final_result')}</p>
  <h2 id="results-heading" class="mt-1 text-2xl font-semibold">
    {#if outcome.kind === 'no_quorum'}
      {i18n.t('tally.no_verdict')}
    {:else if outcome.kind === 'tie'}
      {i18n.t('tally.tie', { choices: outcome.leaders.map(choiceLabel).join(' / ') })}
    {:else}
      {i18n.t('tally.verdict', {
        choice: choiceLabel(outcome.leader),
        percent: i18n.number(Math.round(outcome.pct) / 100, { style: 'percent' })
      })}
    {/if}
  </h2>
  <p class="text-ink-700 mt-1 text-sm">
    {closedAt
      ? i18n.t('tally.counted_summary_closed', { ...counts, date: closedAt })
      : i18n.t('tally.counted_summary', counts)}
  </p>
</section>
