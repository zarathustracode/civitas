<script lang="ts">
  import type { PageData } from './$types';
  import type { OperatorAuditEntry } from '$lib/types/domain';
  import { OPERATOR_AUDIT_PAGE } from '$lib/api/operator';
  import { getI18n } from '$lib/i18n';

  let { data }: { data: PageData } = $props();

  const i18n = getI18n();

  const o = $derived(data.overview);
  const fmtCount = (n: number) => i18n.number(n);
  const fmtWhen = (iso: string) => i18n.date(iso, { dateStyle: 'medium', timeStyle: 'short' });

  // The intro sentence embeds a <time> element: split the translated
  // message around its {when} placeholder and render the element between.
  const intro = $derived(i18n.t('operator.intro').split('{when}'));

  const tiles = $derived([
    {
      id: 'verified',
      label: i18n.t('operator.kpi.verified'),
      value: o.users.verified,
      note: i18n.t('operator.kpi.verified_note', { count: o.users.registered_last_7_days })
    },
    {
      id: 'unverified',
      label: i18n.t('operator.kpi.unverified'),
      value: o.users.unverified,
      note: i18n.t('operator.kpi.unverified_note', {
        deleted: o.users.deleted,
        total: o.users.total
      })
    },
    {
      id: 'delegations',
      label: i18n.t('operator.kpi.delegations'),
      value: o.active_delegations,
      note: i18n.t('operator.kpi.delegations_note')
    },
    {
      id: 'sessions',
      label: i18n.t('operator.kpi.sessions'),
      value: o.active_sessions,
      note: i18n.t('operator.kpi.sessions_note')
    }
  ]);

  const statuses = $derived([
    { id: 'draft', label: i18n.t('operator.status.draft'), value: o.proposals.draft },
    {
      id: 'deliberation',
      label: i18n.t('operator.status.deliberation'),
      value: o.proposals.deliberation
    },
    { id: 'voting', label: i18n.t('operator.status.voting'), value: o.proposals.voting },
    { id: 'closed', label: i18n.t('operator.status.closed'), value: o.proposals.closed }
  ]);

  const turnout = (counted: number, eligible: number) =>
    i18n.number(eligible > 0 ? counted / eligible : 0, {
      style: 'percent',
      maximumFractionDigits: 0
    });

  // Known codes get a translated label; anything newer shows as recorded.
  const translated = (prefix: string, code: string) =>
    i18n.has(`${prefix}.${code}`) ? i18n.t(`${prefix}.${code}`) : code;

  const actionLabel = (action: string) => translated('operator.action', action);

  function detail(e: OperatorAuditEntry): string {
    const m = e.metadata as { from?: string; to?: string; by?: string };
    if (e.action === 'proposal.status_changed') {
      const from = m.from ? translated('operator.stage', m.from) : '?';
      const to = m.to ? translated('operator.stage', m.to) : '?';
      return m.by === 'system'
        ? i18n.t('operator.transition_auto', { from, to })
        : i18n.t('operator.transition', { from, to });
    }
    return '';
  }

  function actor(e: OperatorAuditEntry): string {
    if (e.actor_display_name) return e.actor_display_name;
    return (e.metadata as { by?: string }).by === 'system'
      ? i18n.t('operator.actor_system')
      : i18n.t('operator.actor_unknown');
  }

  // UUIDv7s lead with a timestamp, so rows written together share a prefix;
  // the random tail tells them apart.
  const shortId = (id: string) => `…${id.slice(-8)}`;
  const entityLabel = (type: string) => translated('operator.entity', type);

  function entityHref(e: OperatorAuditEntry): string | null {
    return e.entity_type === 'proposal' ? `/proposals/${e.entity_id}` : null;
  }

  const lastId = $derived(data.audit.at(-1)?.id ?? null);
  const hasOlder = $derived(data.audit.length === OPERATOR_AUDIT_PAGE && lastId !== null);
</script>

<svelte:head>
  <title>{i18n.t('operator.title')} — Civitas</title>
</svelte:head>

<section class="mx-auto max-w-civic px-5 pb-8 pt-16 sm:px-10">
  <p class="mb-4 font-mono text-[11px] uppercase tracking-[0.16em] text-accent-600">
    {i18n.t('operator.eyebrow')}
  </p>
  <h1 class="font-serif text-[clamp(34px,4.4vw,52px)] font-semibold leading-[1.06]">
    {i18n.t('operator.heading')}
  </h1>
  <p class="mt-4 max-w-[62ch] font-serif text-[19px] leading-[1.5] text-ink-600">
    {intro[0]}<time datetime={o.generated_at}>{fmtWhen(o.generated_at)}</time>{intro[1]}
  </p>
</section>

<section class="mx-auto max-w-civic px-5 pb-12 sm:px-10" aria-labelledby="kpi-heading">
  <h2 id="kpi-heading" class="sr-only">{i18n.t('operator.kpi_heading')}</h2>
  <div
    class="grid gap-px overflow-hidden rounded border border-line bg-line sm:grid-cols-2 lg:grid-cols-4"
  >
    {#each tiles as t (t.id)}
      <div class="bg-card px-6 py-5">
        <div class="font-mono text-[10px] uppercase tracking-[0.12em] text-ink-400">{t.label}</div>
        <div class="mt-1.5 font-mono text-[28px] font-medium tabular-nums">{fmtCount(t.value)}</div>
        <p class="mt-1.5 font-mono text-[11px] leading-[1.5] text-ink-400">{t.note}</p>
      </div>
    {/each}
  </div>

  <div
    class="mt-4 flex flex-wrap items-baseline gap-x-8 gap-y-2 font-mono text-[11px] uppercase tracking-[0.1em] text-ink-600"
  >
    <span class="text-ink-400">{i18n.t('operator.proposals')}</span>
    <dl class="flex flex-wrap gap-x-8 gap-y-2">
      {#each statuses as s (s.id)}
        <div class="flex gap-2">
          <dt>{s.label}</dt>
          <dd class="tabular-nums text-ink-900">{fmtCount(s.value)}</dd>
        </div>
      {/each}
    </dl>
  </div>
</section>

<section class="mx-auto max-w-civic px-5 pb-12 sm:px-10" aria-labelledby="voting-heading">
  <h2 id="voting-heading" class="mb-4 font-serif text-[26px] font-semibold">
    {i18n.t('operator.voting_heading')}
  </h2>
  {#if o.voting.length === 0}
    <p
      class="rounded border border-dashed border-line px-6 py-5 font-serif text-[17px] text-ink-600"
    >
      {i18n.t('operator.voting_empty')}
    </p>
  {:else}
    <div class="overflow-x-auto rounded border border-line bg-card">
      <table class="w-full text-left text-[14px]">
        <thead class="font-mono text-[10px] uppercase tracking-[0.12em] text-ink-400">
          <tr class="border-b border-line">
            <th scope="col" class="px-5 py-3 font-normal">{i18n.t('operator.col.proposal')}</th>
            <th scope="col" class="px-5 py-3 font-normal">{i18n.t('operator.col.closes')}</th>
            <th scope="col" class="px-5 py-3 text-right font-normal">
              {i18n.t('operator.col.turnout')}
            </th>
          </tr>
        </thead>
        <tbody class="divide-y divide-line">
          {#each o.voting as p (p.id)}
            <tr>
              <td class="px-5 py-3">
                <a href="/proposals/{p.id}" class="font-medium hover:underline">{p.title}</a>
              </td>
              <td class="px-5 py-3 font-mono text-[12px] tabular-nums text-ink-600">
                {p.voting_ends_at ? fmtWhen(p.voting_ends_at) : '—'}
              </td>
              <td class="px-5 py-3 text-right font-mono text-[12px] tabular-nums">
                {fmtCount(p.counted_voters)} / {fmtCount(p.eligible_voters)}
                <span class="text-ink-400">· {turnout(p.counted_voters, p.eligible_voters)}</span>
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}
</section>

<section class="mx-auto max-w-civic px-5 pb-20 sm:px-10" aria-labelledby="audit-heading">
  <div class="mb-4 flex flex-wrap items-baseline justify-between gap-3">
    <h2 id="audit-heading" class="font-serif text-[26px] font-semibold">
      {data.before ? i18n.t('operator.audit_earlier') : i18n.t('operator.audit_recent')}
    </h2>
    {#if data.before}
      <a
        href="/operator"
        class="font-mono text-[11px] uppercase tracking-[0.1em] text-accent-600 hover:underline"
        >{i18n.t('operator.newest')}</a
      >
    {/if}
  </div>
  {#if data.audit.length === 0}
    <p
      class="rounded border border-dashed border-line px-6 py-5 font-serif text-[17px] text-ink-600"
    >
      {i18n.t('operator.audit_empty')}
    </p>
  {:else}
    <div class="overflow-x-auto rounded border border-line bg-card">
      <table class="w-full text-left text-[13px]">
        <thead class="font-mono text-[10px] uppercase tracking-[0.12em] text-ink-400">
          <tr class="border-b border-line">
            <th scope="col" class="px-5 py-3 font-normal">{i18n.t('operator.col.when')}</th>
            <th scope="col" class="px-5 py-3 font-normal">{i18n.t('operator.col.event')}</th>
            <th scope="col" class="px-5 py-3 font-normal">{i18n.t('operator.col.by')}</th>
            <th scope="col" class="px-5 py-3 font-normal">{i18n.t('operator.col.entity')}</th>
          </tr>
        </thead>
        <tbody class="divide-y divide-line">
          {#each data.audit as e (e.id)}
            {@const href = entityHref(e)}
            <tr>
              <td
                class="whitespace-nowrap px-5 py-2.5 font-mono text-[12px] tabular-nums text-ink-600"
              >
                <time datetime={e.created_at}>{fmtWhen(e.created_at)}</time>
              </td>
              <td class="px-5 py-2.5">
                <span class="font-medium">{actionLabel(e.action)}</span>
                {#if detail(e)}<span class="text-ink-600"> · {detail(e)}</span>{/if}
              </td>
              <td class="px-5 py-2.5">{actor(e)}</td>
              <td class="whitespace-nowrap px-5 py-2.5 font-mono text-[12px] text-ink-600">
                {#if href}
                  <a {href} class="hover:underline" title={e.entity_id}
                    >{entityLabel(e.entity_type)} · {shortId(e.entity_id)}</a
                  >
                {:else}
                  <span title={e.entity_id}
                    >{entityLabel(e.entity_type)} · {shortId(e.entity_id)}</span
                  >
                {/if}
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
    {#if hasOlder}
      <div class="mt-4 text-right">
        <a
          href="/operator?before={lastId}"
          class="font-mono text-[11px] uppercase tracking-[0.1em] text-accent-600 hover:underline"
          >{i18n.t('operator.older')}</a
        >
      </div>
    {/if}
  {/if}
</section>
