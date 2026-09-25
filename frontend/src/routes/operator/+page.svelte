<script lang="ts">
  import type { PageData } from './$types';
  import type { OperatorAuditEntry } from '$lib/types/domain';
  import { OPERATOR_AUDIT_PAGE } from '$lib/api/operator';

  let { data }: { data: PageData } = $props();

  const o = $derived(data.overview);
  const fmtCount = (n: number) => n.toLocaleString('en-US');
  const fmtWhen = (iso: string) =>
    new Date(iso).toLocaleString('en-US', { dateStyle: 'medium', timeStyle: 'short' });

  const tiles = $derived([
    {
      label: 'Verified citizens',
      value: o.users.verified,
      note: `+${fmtCount(o.users.registered_last_7_days)} registered in the last 7 days`
    },
    {
      label: 'Awaiting verification',
      value: o.users.unverified,
      note: `${fmtCount(o.users.deleted)} deleted · ${fmtCount(o.users.total)} ever registered`
    },
    {
      label: 'Active delegations',
      value: o.active_delegations,
      note: 'across all topics'
    },
    {
      label: 'Active sessions',
      value: o.active_sessions,
      note: 'signed-in, unexpired'
    }
  ]);

  const statuses = $derived([
    { label: 'Draft', value: o.proposals.draft },
    { label: 'Deliberation', value: o.proposals.deliberation },
    { label: 'Voting', value: o.proposals.voting },
    { label: 'Closed', value: o.proposals.closed }
  ]);

  const turnout = (counted: number, eligible: number) =>
    eligible > 0 ? Math.round((counted / eligible) * 100) : 0;

  const actionLabel: Record<string, string> = {
    'user.registered': 'Account registered',
    'user.email_verified': 'Email verified',
    'user.phone_verified': 'Phone verified',
    'user.password_changed': 'Password changed',
    'user.deleted': 'Account deleted',
    'session.created': 'Signed in',
    'session.revoked': 'Signed out',
    'topic.created': 'Topic created',
    'proposal.created': 'Proposal created',
    'proposal.status_changed': 'Proposal status changed',
    'vote.cast': 'Vote cast',
    'delegation.created': 'Delegation created',
    'delegation.revoked': 'Delegation revoked',
    'comment.posted': 'Comment posted',
    'comment.edited': 'Comment edited',
    'comment.deleted': 'Comment deleted',
    'comment.hidden': 'Comment hidden'
  };

  function detail(e: OperatorAuditEntry): string {
    const m = e.metadata as { from?: string; to?: string; by?: string };
    if (e.action === 'proposal.status_changed') {
      return `${m.by === 'system' ? 'auto: ' : ''}${m.from ?? '?'} → ${m.to ?? '?'}`;
    }
    return '';
  }

  function actor(e: OperatorAuditEntry): string {
    if (e.actor_display_name) return e.actor_display_name;
    return (e.metadata as { by?: string }).by === 'system' ? 'system' : 'unknown';
  }

  // UUIDv7s lead with a timestamp, so rows written together share a prefix;
  // the random tail tells them apart.
  const shortId = (id: string) => `…${id.slice(-8)}`;

  function entityHref(e: OperatorAuditEntry): string | null {
    return e.entity_type === 'proposal' ? `/proposals/${e.entity_id}` : null;
  }

  const lastId = $derived(data.audit.at(-1)?.id ?? null);
  const hasOlder = $derived(data.audit.length === OPERATOR_AUDIT_PAGE && lastId !== null);
</script>

<svelte:head>
  <title>Operator dashboard — Civitas</title>
</svelte:head>

<section class="mx-auto max-w-civic px-5 pb-8 pt-16 sm:px-10">
  <p class="mb-4 font-mono text-[11px] uppercase tracking-[0.16em] text-accent-600">
    § Operator · read-only
  </p>
  <h1 class="font-serif text-[clamp(34px,4.4vw,52px)] font-semibold leading-[1.06]">
    Deployment overview
  </h1>
  <p class="mt-4 max-w-[62ch] font-serif text-[19px] leading-[1.5] text-ink-600">
    What is happening on this instance, as of
    <time datetime={o.generated_at}>{fmtWhen(o.generated_at)}</time>. Nothing here can change
    anything, and how anyone voted never appears.
  </p>
</section>

<section class="mx-auto max-w-civic px-5 pb-12 sm:px-10" aria-labelledby="kpi-heading">
  <h2 id="kpi-heading" class="sr-only">Headline figures</h2>
  <div
    class="grid gap-px overflow-hidden rounded border border-line bg-line sm:grid-cols-2 lg:grid-cols-4"
  >
    {#each tiles as t (t.label)}
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
    <span class="text-ink-400">Proposals</span>
    <dl class="flex flex-wrap gap-x-8 gap-y-2">
      {#each statuses as s (s.label)}
        <div class="flex gap-2">
          <dt>{s.label}</dt>
          <dd class="tabular-nums text-ink-900">{fmtCount(s.value)}</dd>
        </div>
      {/each}
    </dl>
  </div>
</section>

<section class="mx-auto max-w-civic px-5 pb-12 sm:px-10" aria-labelledby="voting-heading">
  <h2 id="voting-heading" class="mb-4 font-serif text-[26px] font-semibold">Voting now</h2>
  {#if o.voting.length === 0}
    <p
      class="rounded border border-dashed border-line px-6 py-5 font-serif text-[17px] text-ink-600"
    >
      No proposal is in its voting window.
    </p>
  {:else}
    <div class="overflow-x-auto rounded border border-line bg-card">
      <table class="w-full text-left text-[14px]">
        <thead class="font-mono text-[10px] uppercase tracking-[0.12em] text-ink-400">
          <tr class="border-b border-line">
            <th scope="col" class="px-5 py-3 font-normal">Proposal</th>
            <th scope="col" class="px-5 py-3 font-normal">Closes</th>
            <th scope="col" class="px-5 py-3 text-right font-normal">Turnout</th>
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
                <span class="text-ink-400">· {turnout(p.counted_voters, p.eligible_voters)}%</span>
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
      {data.before ? 'Earlier activity' : 'Recent activity'}
    </h2>
    {#if data.before}
      <a
        href="/operator"
        class="font-mono text-[11px] uppercase tracking-[0.1em] text-accent-600 hover:underline"
        >← Newest</a
      >
    {/if}
  </div>
  {#if data.audit.length === 0}
    <p
      class="rounded border border-dashed border-line px-6 py-5 font-serif text-[17px] text-ink-600"
    >
      No recorded events.
    </p>
  {:else}
    <div class="overflow-x-auto rounded border border-line bg-card">
      <table class="w-full text-left text-[13px]">
        <thead class="font-mono text-[10px] uppercase tracking-[0.12em] text-ink-400">
          <tr class="border-b border-line">
            <th scope="col" class="px-5 py-3 font-normal">When</th>
            <th scope="col" class="px-5 py-3 font-normal">Event</th>
            <th scope="col" class="px-5 py-3 font-normal">By</th>
            <th scope="col" class="px-5 py-3 font-normal">Entity</th>
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
                <span class="font-medium">{actionLabel[e.action] ?? e.action}</span>
                {#if detail(e)}<span class="text-ink-600"> · {detail(e)}</span>{/if}
              </td>
              <td class="px-5 py-2.5">{actor(e)}</td>
              <td class="whitespace-nowrap px-5 py-2.5 font-mono text-[12px] text-ink-600">
                {#if href}
                  <a {href} class="hover:underline" title={e.entity_id}
                    >{e.entity_type} · {shortId(e.entity_id)}</a
                  >
                {:else}
                  <span title={e.entity_id}>{e.entity_type} · {shortId(e.entity_id)}</span>
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
          >Older events →</a
        >
      </div>
    {/if}
  {/if}
</section>
