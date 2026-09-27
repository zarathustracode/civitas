<script lang="ts">
  import type { Proposal } from '$lib/types/domain';
  import StatusBadge from './StatusBadge.svelte';
  import { getI18n } from '$lib/i18n';

  let { proposal }: { proposal: Proposal } = $props();

  const i18n = getI18n();

  const votingClosesIn = $derived.by(() => {
    if (proposal.status !== 'voting' || !proposal.voting_ends_at) return null;
    const ends = new Date(proposal.voting_ends_at);
    const ms = ends.getTime() - Date.now();
    if (ms <= 0) return i18n.t('proposal.card.closing');
    const days = Math.floor(ms / 86_400_000);
    const hours = Math.floor((ms % 86_400_000) / 3_600_000);
    if (days > 0) return i18n.t('proposal.card.closes_in_days', { days, hours });
    return i18n.t('proposal.card.closes_in_hours', { hours });
  });
</script>

<a
  href="/proposals/{proposal.id}"
  class="block rounded border border-line bg-card p-5 transition-colors hover:border-ink-400"
>
  <div class="flex items-start justify-between gap-3">
    <h3 class="font-serif text-[20px] font-semibold leading-[1.2] text-ink-900">
      {proposal.title}
    </h3>
    <StatusBadge status={proposal.status} />
  </div>
  <p class="mt-2 font-serif text-[15px] leading-[1.45] text-ink-600">{proposal.summary}</p>
  {#if votingClosesIn}
    <p class="mt-3 font-mono text-[11px] uppercase tracking-[0.08em] text-ink-400">
      {votingClosesIn}
    </p>
  {/if}
</a>
