<script lang="ts">
  import type { ProposalStatus } from '$lib/types/domain';
  import { getI18n } from '$lib/i18n';
  let { status }: { status: ProposalStatus } = $props();

  const i18n = getI18n();

  const meta: Record<ProposalStatus, { cls: string; dot: string }> = {
    draft: { cls: 'border-ink-400 text-ink-400', dot: 'bg-ink-400' },
    deliberation: { cls: 'border-ochre-600 text-ochre-600', dot: 'bg-ochre-600' },
    voting: { cls: 'border-affirm-600 text-affirm-600', dot: 'bg-affirm-600' },
    closed: { cls: 'border-ink-400 text-ink-400', dot: 'bg-ink-400' }
  };
  const m = $derived(meta[status]);
</script>

<span
  class="inline-flex items-center gap-1.5 rounded-full border px-2.5 py-1 font-mono text-[10px] uppercase tracking-[0.1em] {m.cls}"
>
  <span class="h-1.5 w-1.5 rounded-full {m.dot}" aria-hidden="true"></span>{i18n.t(
    `common.status.${status}`
  )}
</span>
