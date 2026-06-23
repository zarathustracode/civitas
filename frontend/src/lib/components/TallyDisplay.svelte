<script lang="ts">
  import type { Tally } from '$lib/types/domain';

  let { tally, live = false }: { tally: Tally; live?: boolean } = $props();

  const t = $derived.by(() => {
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
      ? n.toLocaleString('en-US')
      : n.toLocaleString('en-US', { minimumFractionDigits: 1, maximumFractionDigits: 1 });

  const turnout = $derived(
    tally.eligible_voters > 0 ? Math.round((tally.counted_voters / tally.eligible_voters) * 100) : 0
  );

  const rows = $derived([
    { label: 'Yes', color: 'text-affirm-600', bar: 'bg-affirm-600', val: t.yes, pct: t.yesPct },
    { label: 'No', color: 'text-oppose-600', bar: 'bg-oppose-600', val: t.no, pct: t.noPct },
    {
      label: 'Abstain',
      color: 'text-neutral-600',
      bar: 'bg-neutral-600',
      val: t.abstain,
      pct: t.abstainPct
    }
  ]);
</script>

<section aria-labelledby="tally-heading">
  <div class="mb-4 flex items-center justify-between">
    <div id="tally-heading" class="font-mono text-[10px] uppercase tracking-[0.16em] text-ink-400">
      Live tally
    </div>
    {#if live}
      <div
        class="inline-flex items-center gap-1.5 font-mono text-[10px] uppercase tracking-[0.1em] text-affirm-600"
      >
        <span
          class="h-1.5 w-1.5 rounded-full bg-affirm-600"
          style="animation:blink 2s steps(1) infinite;"
          aria-hidden="true"
        ></span>Counting
      </div>
    {/if}
  </div>

  {#if t.counted === 0}
    <p class="text-[13px] text-ink-600">No votes counted yet.</p>
  {:else}
    <div class="flex flex-col gap-3.5">
      {#each rows as r (r.label)}
        <div>
          <div class="mb-1.5 flex items-baseline justify-between">
            <span class="text-[14px] font-semibold {r.color}">{r.label}</span>
            <span class="font-mono text-[12px] tabular-nums text-ink-600"
              >{fmt(r.val)} · {Math.round(r.pct)}%</span
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
      {tally.counted_voters.toLocaleString('en-US')} / {tally.eligible_voters.toLocaleString(
        'en-US'
      )}
      eligible
    </span>
    <span class="font-mono text-[11px] uppercase tracking-[0.06em] text-ink-400"
      >{turnout}% turnout</span
    >
  </div>
</section>
