<script lang="ts">
  import type { AuditEntry } from '$lib/types/domain';
  import { getI18n } from '$lib/i18n';

  let { entries }: { entries: AuditEntry[] } = $props();

  const i18n = getI18n();

  /** Known actions get a label; anything newer shows its code. */
  function actionLabel(action: string): string {
    const key = `audit.action.${action.replace(/\./g, '_')}`;
    return i18n.has(key) ? i18n.t(key) : action;
  }

  /** A proposal status as the timeline names it; unknown values as written. */
  function statusName(status: string | undefined): string {
    if (!status) return '?';
    const key = `audit.status.${status}`;
    return i18n.has(key) ? i18n.t(key) : status;
  }

  const fmt = (iso: string) => i18n.date(iso, { dateStyle: 'medium', timeStyle: 'short' });

  function describe(entry: AuditEntry): string {
    if (entry.action === 'proposal.status_changed') {
      const m = entry.metadata as { from?: string; to?: string; by?: string };
      const params = { from: statusName(m.from), to: statusName(m.to) };
      return m.by === 'system'
        ? i18n.t('audit.transition_auto', params)
        : i18n.t('audit.transition', params);
    }
    return '';
  }

  function actorLabel(entry: AuditEntry): string {
    if (entry.actor_display_name) return i18n.t('audit.by', { name: entry.actor_display_name });
    const m = entry.metadata as { by?: string };
    if (m.by === 'system') return i18n.t('audit.by_system');
    return i18n.t('audit.by_unknown');
  }
</script>

<details class="overflow-hidden rounded border border-line bg-card">
  <summary
    class="cursor-pointer select-none px-4 py-3 font-mono text-[11px] uppercase tracking-[0.12em] text-ink-600"
  >
    {i18n.t('audit.title', { count: entries.length })}
  </summary>
  {#if entries.length === 0}
    <p class="px-4 pb-4 text-[13px] text-ink-600">{i18n.t('audit.empty')}</p>
  {:else}
    <ol class="divide-y divide-line px-4 pb-3 text-[13px]">
      {#each entries as e (e.id)}
        <li class="grid gap-1 py-2.5 sm:grid-cols-[12rem_1fr_auto] sm:items-baseline sm:gap-3">
          <span class="font-medium">{actionLabel(e.action)}</span>
          <span class="text-ink-600">{describe(e)}</span>
          <span class="font-mono text-[11px] text-ink-400">
            <span>{actorLabel(e)}</span> · <span class="tabular-nums">{fmt(e.created_at)}</span>
          </span>
        </li>
      {/each}
    </ol>
  {/if}
</details>
