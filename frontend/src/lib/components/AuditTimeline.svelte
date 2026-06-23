<script lang="ts">
  import type { AuditEntry } from '$lib/types/domain';

  let { entries }: { entries: AuditEntry[] } = $props();

  const actionLabel: Record<string, string> = {
    'proposal.created': 'Proposal created',
    'proposal.status_changed': 'Status changed'
  };

  const fmt = (iso: string) =>
    new Date(iso).toLocaleString(undefined, {
      dateStyle: 'medium',
      timeStyle: 'short'
    });

  function describe(entry: AuditEntry): string {
    if (entry.action === 'proposal.status_changed') {
      const m = entry.metadata as { from?: string; to?: string; by?: string };
      const prefix = m.by === 'system' ? 'auto: ' : '';
      return `${prefix}${m.from ?? '?'} → ${m.to ?? '?'}`;
    }
    return '';
  }

  function actorLabel(entry: AuditEntry): string {
    if (entry.actor_display_name) return entry.actor_display_name;
    const m = entry.metadata as { by?: string };
    if (m.by === 'system') return 'system';
    return 'unknown';
  }
</script>

<details class="overflow-hidden rounded border border-line bg-card">
  <summary
    class="cursor-pointer select-none px-4 py-3 font-mono text-[11px] uppercase tracking-[0.12em] text-ink-600"
  >
    Audit timeline · {entries.length}
  </summary>
  {#if entries.length === 0}
    <p class="px-4 pb-4 text-[13px] text-ink-600">No recorded events on this proposal yet.</p>
  {:else}
    <ol class="divide-y divide-line px-4 pb-3 text-[13px]">
      {#each entries as e (e.id)}
        <li class="grid gap-1 py-2.5 sm:grid-cols-[12rem_1fr_auto] sm:items-baseline sm:gap-3">
          <span class="font-medium">{actionLabel[e.action] ?? e.action}</span>
          <span class="text-ink-600">{describe(e)}</span>
          <span class="font-mono text-[11px] text-ink-400">
            <span>by {actorLabel(e)}</span> · <span class="tabular-nums">{fmt(e.created_at)}</span>
          </span>
        </li>
      {/each}
    </ol>
  {/if}
</details>
