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
    // For events with no specific metadata to render, leave the description
    // cell empty — the action label cell is enough.
    return '';
  }

  function actorLabel(entry: AuditEntry): string {
    if (entry.actor_display_name) return entry.actor_display_name;
    const m = entry.metadata as { by?: string };
    if (m.by === 'system') return 'system';
    return 'unknown';
  }
</script>

<details class="rounded-lg border border-ink-200 bg-white">
  <summary class="cursor-pointer select-none p-3 text-sm font-semibold">
    Audit timeline ({entries.length})
  </summary>
  {#if entries.length === 0}
    <p class="px-3 pb-3 text-sm text-ink-600">No recorded events on this proposal yet.</p>
  {:else}
    <ol class="divide-y divide-ink-100 px-3 pb-3 text-sm">
      {#each entries as e (e.id)}
        <li class="grid gap-1 py-2 sm:grid-cols-[10rem_1fr_auto] sm:items-baseline sm:gap-3">
          <span class="font-medium">{actionLabel[e.action] ?? e.action}</span>
          <span class="text-ink-700">{describe(e)}</span>
          <span class="text-xs text-ink-600">
            <span>by {actorLabel(e)}</span> · <span class="tabular-nums">{fmt(e.created_at)}</span>
          </span>
        </li>
      {/each}
    </ol>
  {/if}
</details>
