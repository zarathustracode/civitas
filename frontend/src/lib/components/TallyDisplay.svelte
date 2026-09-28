<script lang="ts">
  import type { Tally } from '$lib/types/domain';
  import { getI18n } from '$lib/i18n';

  let { tally, live = false }: { tally: Tally; live?: boolean } = $props();

  const i18n = getI18n();

  const totals = $derived.by(() => {
    const yes = parseFloat(tally.yes);
    const no = parseFloat(tally.no);
    const abstain = parseFloat(tally.abstain);
    const counted = yes + no + abstain;
    const pct = (n: number) => (counted > 0 ? (n / counted) * 100 : 0);
    return {
      yes,
      no,
      abstain,
      counted,
      yesPct: pct(yes),
      noPct: pct(no),
      abstainPct: pct(abstain)
    };
  });

  const fmt = (n: number) =>
    Number.isInteger(n)
      ? i18n.number(n)
      : i18n.number(n, { minimumFractionDigits: 1, maximumFractionDigits: 1 });

  /** A whole percentage, e.g. 42 → "42%" ("42 %" in Spanish). */
  const percent = (n: number) => i18n.number(Math.round(n) / 100, { style: 'percent' });

  const turnout = $derived(
    tally.eligible_voters > 0 ? Math.round((tally.counted_voters / tally.eligible_voters) * 100) : 0
  );

  const rows = $derived([
    {
      key: 'yes',
      label: i18n.t('common.choice.yes'),
      color: 'text-affirm-600',
      bar: 'bg-affirm-600',
      val: totals.yes,
      pct: totals.yesPct
    },
    {
      key: 'no',
      label: i18n.t('common.choice.no'),
      color: 'text-oppose-600',
      bar: 'bg-oppose-600',
      val: totals.no,
      pct: totals.noPct
    },
    {
      key: 'abstain',
      label: i18n.t('common.choice.abstain'),
      color: 'text-neutral-600',
      bar: 'bg-neutral-600',
      val: totals.abstain,
      pct: totals.abstainPct
    }
  ]);
</script>

<section aria-labelledby="tally-heading">
  <div class="mb-4 flex items-center justify-between">
    <div id="tally-heading" class="font-mono text-[10px] uppercase tracking-[0.16em] text-ink-400">
      {i18n.t('tally.live_heading')}
    </div>
    {#if live}
      <div
        class="inline-flex items-center gap-1.5 font-mono text-[10px] uppercase tracking-[0.1em] text-affirm-600"
      >
        <span
          class="h-1.5 w-1.5 rounded-full bg-affirm-600"
          style="animation:blink 2s steps(1) infinite;"
          aria-hidden="true"
        ></span>{i18n.t('tally.counting')}
      </div>
    {/if}
  </div>

  {#if totals.counted === 0}
    <p class="text-[13px] text-ink-600">{i18n.t('tally.no_votes_yet')}</p>
  {:else}
    <div class="flex flex-col gap-3.5">
      {#each rows as r (r.key)}
        <div>
          <div class="mb-1.5 flex items-baseline justify-between">
            <span class="text-[14px] font-semibold {r.color}">{r.label}</span>
            <span class="font-mono text-[12px] tabular-nums text-ink-600"
              >{fmt(r.val)} · {percent(r.pct)}</span
            >
          </div>
          <div class="h-2 overflow-hidden rounded-full bg-ink-100">
            <div
              class="h-full rounded-full {r.bar} transition-[width] duration-[250ms]"
              style="width:{r.pct}%"
            ></div>
          </div>
        </div>
      {/each}
    </div>
  {/if}

  <div class="mt-[18px] flex items-center justify-between border-t border-line pt-3.5">
    <span class="font-mono text-[11px] tabular-nums text-ink-600">
      {i18n.t('tally.eligible_count', {
        counted: tally.counted_voters,
        eligible: tally.eligible_voters
      })}
    </span>
    <span class="font-mono text-[11px] uppercase tracking-[0.06em] text-ink-400"
      >{i18n.t('tally.turnout', { percent: percent(turnout) })}</span
    >
  </div>
</section>
