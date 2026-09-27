<script lang="ts">
  import { onMount } from 'svelte';
  import type { PageData } from './$types';
  import type { ProposalStatus, ProposalListItem } from '$lib/types/domain';
  import { getI18n } from '$lib/i18n';

  let { data }: { data: PageData } = $props();

  const i18n = getI18n();

  type Filter = 'all' | ProposalStatus;
  let filter = $state<Filter>('all');
  let now = $state(0);

  onMount(() => {
    now = Date.now();
    const t = setInterval(() => (now = Date.now()), 30_000);
    return () => clearInterval(t);
  });

  const topicName = (id: string) =>
    data.topics.find((t) => t.id === id)?.name ?? i18n.t('docket.topic_fallback');

  // Filing number: rank by creation order (oldest = 1), stable across filters.
  const filingNumber = $derived.by(() => {
    const map = new Map<string, number>();
    [...data.items]
      .sort((a, b) => new Date(a.created_at).getTime() - new Date(b.created_at).getTime())
      .forEach((p, i) => map.set(p.id, i + 1));
    return map;
  });

  const counts = $derived.by(() => {
    const c = { all: data.items.length, draft: 0, deliberation: 0, voting: 0, closed: 0 };
    for (const p of data.items) c[p.status] += 1;
    return c;
  });

  const statusMeta: Record<ProposalStatus, { text: string; bg: string }> = {
    voting: { text: 'text-accent-600', bg: 'bg-accent-600' },
    deliberation: { text: 'text-ochre-600', bg: 'bg-ochre-600' },
    closed: { text: 'text-ink-400', bg: 'bg-ink-400' },
    draft: { text: 'text-ink-400', bg: 'bg-ink-400' }
  };

  function endsLabel(iso: string | null): string {
    if (!iso) return i18n.t('docket.voting_open');
    const ms = new Date(iso).getTime() - (now || Date.now());
    if (ms <= 0) return i18n.t('docket.closing');
    const days = Math.floor(ms / 86_400_000);
    const hours = Math.floor((ms % 86_400_000) / 3_600_000);
    if (days > 0) return i18n.t('docket.days_left', { count: days });
    return i18n.t('docket.hours_left', { count: hours });
  }

  interface Row {
    item: ProposalListItem;
    num: number;
    statusLabel: string;
    statusText: string;
    railBg: string;
    topic: string;
    commentsLabel: string;
    hasBar: boolean;
    yesPct: number;
    noPct: number;
    absPct: number;
    yesLabel: string;
    rightMeta: string;
    hasChip: boolean;
    chipLabel: string;
    chipText: string;
    chipBorder: string;
  }

  const rows = $derived.by<Row[]>(() => {
    return data.items
      .filter((p) => filter === 'all' || p.status === filter)
      .map((p) => {
        const meta = statusMeta[p.status];
        const yes = parseFloat(p.yes);
        const no = parseFloat(p.no);
        const abstain = parseFloat(p.abstain);
        const counted = yes + no + abstain;
        const pct = (n: number) => (counted > 0 ? (n / counted) * 100 : 0);
        const isVoting = p.status === 'voting';
        const isClosed = p.status === 'closed';
        const isDelib = p.status === 'deliberation';
        const passed = yes > no;
        const comments = i18n.t('docket.comment_count', { count: p.comment_count });
        return {
          item: p,
          num: filingNumber.get(p.id) ?? 0,
          statusLabel: i18n.t(`common.status.${p.status}`),
          statusText: meta.text,
          railBg: meta.bg,
          topic: topicName(p.topic_id),
          commentsLabel: p.comment_count > 0 ? comments : i18n.t('docket.comments_not_open'),
          hasBar: isVoting || isClosed,
          yesPct: pct(yes),
          noPct: pct(no),
          absPct: pct(abstain),
          yesLabel: i18n.t('docket.yes_share', {
            percent: i18n.number(Math.round(pct(yes)) / 100, { style: 'percent' })
          }),
          rightMeta: isVoting
            ? endsLabel(p.voting_ends_at)
            : counted === 0
              ? i18n.t('docket.no_votes')
              : passed
                ? i18n.t('docket.passed')
                : i18n.t('docket.failed'),
          hasChip: isDelib || p.status === 'draft',
          chipLabel: isDelib ? comments : i18n.t('common.status.draft'),
          chipText: isDelib ? 'text-ochre-600' : 'text-ink-400',
          chipBorder: isDelib ? 'border-ochre-600' : 'border-ink-400'
        };
      });
  });

  const tabs: { value: Filter; label: string; count: number }[] = $derived([
    { value: 'all', label: i18n.t('docket.filter.all'), count: counts.all },
    { value: 'voting', label: i18n.t('docket.filter.voting'), count: counts.voting },
    {
      value: 'deliberation',
      label: i18n.t('docket.filter.deliberation'),
      count: counts.deliberation
    },
    { value: 'closed', label: i18n.t('docket.filter.closed'), count: counts.closed },
    { value: 'draft', label: i18n.t('docket.filter.draft'), count: counts.draft }
  ]);
</script>

<svelte:head>
  <title>{i18n.t('docket.title')} — Civitas</title>
</svelte:head>

<!-- HEADER -->
<section class="mx-auto max-w-civic px-5 pb-7 pt-14 sm:px-10">
  <div
    class="mb-3.5 font-mono text-[11px] uppercase tracking-[0.2em] text-ink-400"
    style="animation:fadeUp .6s both;"
  >
    {i18n.t('docket.eyebrow')}
  </div>
  <h1
    class="font-serif text-[clamp(40px,5.4vw,60px)] font-semibold leading-[1.04] tracking-[-0.015em]"
    style="animation:fadeUp .6s both .08s;"
  >
    {i18n.t('docket.title')}
  </h1>
  <p
    class="mt-[18px] max-w-[58ch] font-serif text-[20px] leading-[1.5] text-ink-600"
    style="animation:fadeUp .6s both .14s;"
  >
    {i18n.t('docket.intro')}
  </p>
</section>

<!-- FILTERS -->
<section class="mx-auto max-w-civic px-5 pt-2 sm:px-10">
  <div class="flex flex-wrap gap-2 border-b border-line pb-[22px]">
    {#each tabs as tab (tab.value)}
      <button
        type="button"
        onclick={() => (filter = tab.value)}
        aria-pressed={filter === tab.value}
        class="cursor-pointer rounded-full border px-[15px] py-2 font-mono text-[11px] uppercase tracking-[0.08em] transition-colors {filter ===
        tab.value
          ? 'border-ink-900 bg-ink-900 text-white'
          : 'border-line bg-white text-ink-600 hover:border-ink-400'}"
      >
        {tab.label}
        <span class={filter === tab.value ? 'text-[#b9b6aa]' : 'text-ink-400'}
          >{i18n.number(tab.count)}</span
        >
      </button>
    {/each}
  </div>
</section>

<!-- LIST -->
<section class="mx-auto max-w-civic px-5 pb-20 pt-2 sm:px-10">
  {#if rows.length === 0}
    <p
      class="mt-8 rounded-md border border-dashed border-line px-7 py-7 font-serif text-[18px] text-ink-600"
    >
      {i18n.t('docket.empty')}
    </p>
  {:else}
    {#each rows as row (row.item.id)}
      <a
        href="/proposals/{row.item.id}"
        class="flex flex-wrap items-stretch border-b border-line text-ink-900 transition-colors hover:bg-[#faf8f2]"
      >
        <span class="w-1.5 self-stretch {row.railBg}" aria-hidden="true"></span>
        <div
          class="flex w-[78px] flex-none items-start py-7 pl-[22px] font-mono text-[26px] font-medium tabular-nums tracking-[-0.02em] text-ink-400"
        >
          {row.num}
        </div>
        <div class="min-w-[240px] flex-1 py-7 pl-3.5 pr-7">
          <div class="mb-2.5 flex flex-wrap items-center gap-3">
            <span
              class="inline-flex items-center gap-2 font-mono text-[10px] uppercase tracking-[0.12em] {row.statusText}"
            >
              <span class="h-1.5 w-1.5 rounded-full {row.railBg}"></span>{row.statusLabel}
            </span>
            <span class="font-mono text-[10px] uppercase tracking-[0.12em] text-ink-400"
              >{row.topic}</span
            >
          </div>
          <div class="mb-2 font-serif text-[23px] font-semibold leading-[1.18] tracking-[-0.01em]">
            {row.item.title}
          </div>
          <div class="max-w-[60ch] font-serif text-[16px] leading-[1.45] text-ink-600">
            {row.item.summary}
          </div>
          <div class="mt-3 font-mono text-[11px] tracking-[0.04em] text-ink-400">
            {row.commentsLabel}
          </div>
        </div>
        <div class="flex w-full flex-none items-center justify-end py-4 sm:w-[220px] sm:py-7">
          {#if row.hasBar}
            <div class="w-[180px]">
              <div class="flex h-2 overflow-hidden rounded-full bg-ink-100">
                <div class="bg-affirm-600" style="width:{row.yesPct}%"></div>
                <div class="bg-oppose-600" style="width:{row.noPct}%"></div>
                <div class="bg-neutral-600" style="width:{row.absPct}%"></div>
              </div>
              <div
                class="mt-2 flex justify-between font-mono text-[11px] tabular-nums text-ink-600"
              >
                <span class="text-affirm-600">{row.yesLabel}</span>
                <span>{row.rightMeta}</span>
              </div>
            </div>
          {:else if row.hasChip}
            <span
              class="inline-flex items-center gap-2 rounded-full border px-[13px] py-[7px] font-mono text-[11px] uppercase tracking-[0.06em] {row.chipText} {row.chipBorder}"
            >
              {row.chipLabel}
            </span>
          {/if}
        </div>
      </a>
    {/each}
  {/if}
</section>
