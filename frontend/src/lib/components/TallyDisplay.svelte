<script lang="ts">
  import type { Tally } from '$lib/types/domain';

  let { tally }: { tally: Tally } = $props();

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

  const fmt = (n: number) => (Number.isInteger(n) ? n.toString() : n.toFixed(2));
</script>

<div class="space-y-3">
  <h3 class="text-lg font-semibold">Tally</h3>

  {#if totals.counted === 0}
    <p class="text-sm text-ink-600">No votes counted yet.</p>
  {:else}
    <div class="space-y-2">
      <div>
        <div class="flex items-baseline justify-between text-sm">
          <span class="font-medium text-affirm-600">Yes</span>
          <span class="tabular-nums text-ink-600">
            {fmt(totals.yes)} ({totals.yesPct.toFixed(0)}%)
          </span>
        </div>
        <div class="h-2 w-full rounded bg-ink-100">
          <div
            class="h-full rounded bg-affirm-600"
            style="width: {totals.yesPct}%"
            aria-hidden="true"
          ></div>
        </div>
      </div>

      <div>
        <div class="flex items-baseline justify-between text-sm">
          <span class="font-medium text-oppose-600">No</span>
          <span class="tabular-nums text-ink-600">
            {fmt(totals.no)} ({totals.noPct.toFixed(0)}%)
          </span>
        </div>
        <div class="h-2 w-full rounded bg-ink-100">
          <div
            class="h-full rounded bg-oppose-600"
            style="width: {totals.noPct}%"
            aria-hidden="true"
          ></div>
        </div>
      </div>

      <div>
        <div class="flex items-baseline justify-between text-sm">
          <span class="font-medium text-neutral-600">Abstain</span>
          <span class="tabular-nums text-ink-600">
            {fmt(totals.abstain)} ({totals.abstainPct.toFixed(0)}%)
          </span>
        </div>
        <div class="h-2 w-full rounded bg-ink-100">
          <div
            class="h-full rounded bg-neutral-600"
            style="width: {totals.abstainPct}%"
            aria-hidden="true"
          ></div>
        </div>
      </div>
    </div>
  {/if}

  <p class="text-xs text-ink-600">
    {tally.counted_voters} of {tally.eligible_voters} eligible voters counted
  </p>
</div>
