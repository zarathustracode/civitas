<script lang="ts">
  import type { ProposalStatus } from '$lib/types/domain';
  let { status }: { status: ProposalStatus } = $props();
  const styleByStatus: Record<ProposalStatus, string> = {
    draft: 'bg-ink-100 text-ink-600',
    deliberation: 'bg-accent-50 text-accent-700',
    voting: 'bg-affirm-600/10 text-affirm-600',
    closed: 'bg-ink-200 text-ink-800'
  };
  const labelByStatus: Record<ProposalStatus, string> = {
    draft: 'Draft',
    deliberation: 'Deliberation',
    voting: 'Voting',
    closed: 'Closed'
  };
</script>

<span
  class="inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium {styleByStatus[
    status
  ]}"
>
  {labelByStatus[status]}
</span>
