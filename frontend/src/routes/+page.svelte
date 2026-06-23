<script lang="ts">
  import { onMount } from 'svelte';
  import type { PageData } from './$types';

  let { data }: { data: PageData } = $props();

  const reduceMotion =
    typeof window !== 'undefined' && window.matchMedia('(prefers-reduced-motion: reduce)').matches;

  // Real targets from the featured proposal's live tally.
  const yesTarget = $derived(data.featuredTally ? parseFloat(data.featuredTally.yes) : 0);
  const noTarget = $derived(data.featuredTally ? parseFloat(data.featuredTally.no) : 0);
  const abstainTarget = $derived(data.featuredTally ? parseFloat(data.featuredTally.abstain) : 0);

  // Animated display values (ramp toward the real targets).
  let dispYes = $state(0);
  let dispNo = $state(0);
  let statVoting = $state(0);
  let statDelib = $state(0);
  let statTopics = $state(0);
  let now = $state(0);

  const total = $derived(dispYes + dispNo + abstainTarget);
  const yesPct = $derived(total > 0 ? (dispYes / total) * 100 : 0);
  const noPct = $derived(total > 0 ? (dispNo / total) * 100 : 0);
  const turnout = $derived(
    data.featuredTally && data.featuredTally.eligible_voters > 0
      ? Math.round((data.featuredTally.counted_voters / data.featuredTally.eligible_voters) * 100)
      : 0
  );

  const fmt = (n: number) => {
    const r = Math.round(n * 10) / 10;
    return Number.isInteger(r)
      ? r.toLocaleString('en-US')
      : r.toLocaleString('en-US', { minimumFractionDigits: 1, maximumFractionDigits: 1 });
  };
  const whole = (n: number) => Math.round(n).toLocaleString('en-US');

  const endsAt = $derived(
    data.featured?.voting_ends_at ? new Date(data.featured.voting_ends_at).getTime() : 0
  );
  const countdown = $derived.by(() => {
    if (!endsAt || !now) return null;
    let ms = Math.max(0, endsAt - now);
    const days = Math.floor(ms / 86_400_000);
    ms -= days * 86_400_000;
    const hrs = Math.floor(ms / 3_600_000);
    ms -= hrs * 3_600_000;
    const mins = Math.floor(ms / 60_000);
    ms -= mins * 60_000;
    const secs = Math.floor(ms / 1000);
    const pad = (n: number) => String(n).padStart(2, '0');
    return { days, h: pad(hrs), m: pad(mins), s: pad(secs) };
  });

  onMount(() => {
    now = Date.now();
    if (reduceMotion) {
      dispYes = yesTarget;
      dispNo = noTarget;
      statVoting = data.stats.votingOpen;
      statDelib = data.stats.inDeliberation;
      statTopics = data.stats.topics;
    }
    const step = (a: number, b: number) => (Math.abs(b - a) < 0.6 ? b : a + (b - a) * 0.09);
    const anim = reduceMotion
      ? null
      : setInterval(() => {
          dispYes = step(dispYes, yesTarget);
          dispNo = step(dispNo, noTarget);
          statVoting = step(statVoting, data.stats.votingOpen);
          statDelib = step(statDelib, data.stats.inDeliberation);
          statTopics = step(statTopics, data.stats.topics);
        }, 30);
    const clock = setInterval(() => {
      now = Date.now();
    }, 1000);
    return () => {
      if (anim) clearInterval(anim);
      clearInterval(clock);
    };
  });
</script>

<svelte:head>
  <title>Civitas — direct democracy with delegation</title>
</svelte:head>

<!-- HERO -->
<section
  class="mx-auto grid max-w-civic items-center gap-14 px-5 pb-14 pt-20 sm:px-10 lg:grid-cols-[1.4fr_1fr]"
>
  <div>
    <div class="mb-6 font-mono text-[11px] uppercase tracking-[0.22em] text-ink-400">
      Open civic infrastructure
    </div>
    <h1
      class="font-serif text-[clamp(44px,6vw,76px)] font-semibold leading-[1.02] tracking-[-0.02em]"
    >
      Vote directly —<br /><span class="font-medium italic text-ink-600"
        >or trust someone who will.</span
      >
    </h1>
    <p class="mt-7 max-w-[52ch] font-serif text-[21px] leading-[1.55] text-ink-600">
      Civitas is a platform for direct democracy with optional, per-topic delegation. Verified
      citizens decide policy together — every tally public, every change reversible, every weight
      traceable to the person who carried it.
    </p>
    <div class="mt-8 flex flex-wrap gap-3">
      <a
        href="/proposals"
        class="inline-flex items-center gap-2 rounded-[3px] bg-accent-600 px-[22px] py-[14px] text-[15px] font-semibold text-white transition-colors hover:bg-accent-700"
      >
        Browse proposals <span class="font-mono">→</span>
      </a>
      <a
        href="/delegations"
        class="inline-flex items-center rounded-[3px] border border-line bg-white px-[22px] py-[14px] text-[15px] font-medium text-ink-900 transition-colors hover:border-ink-900"
      >
        Manage delegations
      </a>
    </div>
  </div>

  <!-- LIVE INSTRUMENT -->
  {#if data.featured}
    <a href="/proposals/{data.featured.id}" class="block text-ink-900">
      <div
        class="rounded-[5px] border border-line bg-card p-6 shadow-[0_18px_40px_-28px_rgba(27,26,20,0.4)]"
      >
        <div class="mb-[18px] flex items-center justify-between">
          <span
            class="inline-flex items-center gap-2 font-mono text-[10px] uppercase tracking-[0.12em] text-accent-600"
          >
            <span
              class="h-[6px] w-[6px] rounded-full bg-accent-600"
              style="animation:blink 2s steps(1) infinite;"
              aria-hidden="true"
            ></span>Voting now
          </span>
          <span class="font-mono text-[11px] tabular-nums tracking-[0.06em] text-ink-400">
            {#if countdown}
              closes {countdown.days}d {countdown.h}:{countdown.m}:{countdown.s}
            {:else}
              voting open
            {/if}
          </span>
        </div>
        {#if data.featuredTopicName}
          <div class="mb-1.5 font-mono text-[11px] uppercase tracking-[0.12em] text-ink-400">
            {data.featuredTopicName}
          </div>
        {/if}
        <div class="mb-5 font-serif text-[24px] font-semibold leading-[1.18]">
          {data.featured.title}
        </div>
        <div class="flex flex-col gap-[11px]">
          <div>
            <div class="mb-[5px] flex justify-between font-mono text-[12px]">
              <span class="font-medium text-affirm-600">Yes</span>
              <span class="tabular-nums text-ink-600">{fmt(dispYes)}</span>
            </div>
            <div class="h-[7px] overflow-hidden rounded-full bg-ink-100">
              <div
                class="h-full rounded-full bg-affirm-600 transition-[width] duration-200"
                style="width:{yesPct}%"
              ></div>
            </div>
          </div>
          <div>
            <div class="mb-[5px] flex justify-between font-mono text-[12px]">
              <span class="font-medium text-oppose-600">No</span>
              <span class="tabular-nums text-ink-600">{fmt(dispNo)}</span>
            </div>
            <div class="h-[7px] overflow-hidden rounded-full bg-ink-100">
              <div
                class="h-full rounded-full bg-oppose-600 transition-[width] duration-200"
                style="width:{noPct}%"
              ></div>
            </div>
          </div>
        </div>
        <div
          class="mt-[18px] flex items-center justify-between border-t border-line pt-[14px] font-mono text-[11px] text-ink-400"
        >
          <span class="tabular-nums">{turnout}% turnout</span>
          <span class="text-accent-600">open proposal →</span>
        </div>
      </div>
    </a>
  {:else}
    <a href="/proposals" class="block text-ink-900">
      <div class="rounded-[5px] border border-line bg-card p-6">
        <div class="mb-[18px] font-mono text-[10px] uppercase tracking-[0.12em] text-ink-400">
          The docket
        </div>
        <div class="mb-2 font-serif text-[24px] font-semibold leading-[1.18]">
          No proposals are open for voting right now.
        </div>
        <p class="font-serif text-[17px] leading-[1.5] text-ink-600">
          Browse the docket to follow what is in deliberation and what has closed.
        </p>
        <div
          class="mt-[18px] flex items-center justify-between border-t border-line pt-[14px] font-mono text-[11px] text-ink-400"
        >
          <span class="tabular-nums">{whole(statDelib)} in deliberation</span>
          <span class="text-accent-600">open docket →</span>
        </div>
      </div>
    </a>
  {/if}
</section>

<!-- STAT BAND -->
<section class="border-y border-line bg-card">
  <div class="mx-auto flex max-w-civic flex-wrap items-baseline gap-12 px-5 py-[30px] sm:px-10">
    <div class="flex items-baseline gap-3">
      <span class="font-mono text-[30px] font-medium tabular-nums">{whole(statVoting)}</span>
      <span class="font-mono text-[11px] uppercase tracking-[0.1em] text-ink-400">
        proposals open
      </span>
    </div>
    <div class="flex items-baseline gap-3">
      <span class="font-mono text-[30px] font-medium tabular-nums">{whole(statDelib)}</span>
      <span class="font-mono text-[11px] uppercase tracking-[0.1em] text-ink-400">
        in deliberation
      </span>
    </div>
    <div class="flex items-baseline gap-3">
      <span class="font-mono text-[30px] font-medium tabular-nums">{whole(statTopics)}</span>
      <span class="font-mono text-[11px] uppercase tracking-[0.1em] text-ink-400"> topics </span>
    </div>
  </div>
</section>

<!-- HOW IT WORKS -->
<section class="mx-auto max-w-civic px-5 py-20 sm:px-10">
  <div class="mb-9 font-mono text-[11px] uppercase tracking-[0.2em] text-ink-400">How it works</div>
  <div class="grid gap-9 md:grid-cols-3">
    <div>
      <div class="mb-4 font-mono text-[13px] text-accent-600">01</div>
      <h3 class="mb-2.5 font-serif text-[24px] font-semibold tracking-[-0.01em]">
        Register &amp; verify
      </h3>
      <p class="font-serif text-[17px] leading-[1.55] text-ink-600">
        Verify your email and you become eligible to vote and to take part in structured
        deliberation. Minimal data, clear retention, easy deletion.
      </p>
    </div>
    <div>
      <div class="mb-4 font-mono text-[13px] text-accent-600">02</div>
      <h3 class="mb-2.5 font-serif text-[24px] font-semibold tracking-[-0.01em]">
        Read &amp; deliberate
      </h3>
      <p class="font-serif text-[17px] leading-[1.55] text-ink-600">
        Every proposal carries its full body and an open thread. Argue, question, and surface the
        trade-offs before anything is decided.
      </p>
    </div>
    <div>
      <div class="mb-4 font-mono text-[13px] text-accent-600">03</div>
      <h3 class="mb-2.5 font-serif text-[24px] font-semibold tracking-[-0.01em]">
        Vote or delegate
      </h3>
      <p class="font-serif text-[17px] leading-[1.55] text-ink-600">
        Cast a direct vote, or delegate a topic to someone you trust. Tallies report every weight
        with the chain that carried it.
      </p>
    </div>
  </div>
</section>

<!-- SIGNATURE BAND -->
<section class="bg-band text-band-ink">
  <div class="mx-auto max-w-civic px-5 py-20 sm:px-10">
    <div class="mb-3.5 font-mono text-[11px] uppercase tracking-[0.2em] text-band-mute">
      The mechanism
    </div>
    <h2
      class="mb-2.5 max-w-[18ch] font-serif text-[clamp(32px,4.4vw,52px)] font-medium leading-[1.06] tracking-[-0.015em]"
    >
      A direct vote always wins.
    </h2>
    <p class="mb-14 max-w-[58ch] font-serif text-[20px] leading-[1.55] text-[#b9b6aa]">
      Delegated weight flows along a chain of trust until it reaches someone who votes directly.
      Every link is visible. The moment you vote yourself, your weight leaves the chain.
    </p>

    <!-- CHAIN (illustrative) -->
    <div class="flex min-w-0 items-start" aria-hidden="true">
      <div class="w-[184px] flex-none text-center">
        <div
          class="mx-auto flex h-16 w-16 items-center justify-center rounded-full border border-[rgba(125,151,255,0.5)] bg-[rgba(125,151,255,0.14)] font-serif text-[18px] font-semibold text-[#cdd6ff]"
        >
          You
        </div>
        <div class="mt-[13px] text-[14px] font-semibold">You</div>
        <div class="mt-1 font-mono text-[10px] uppercase tracking-[0.1em] text-[#8f8c80]">
          1.0 weight
        </div>
      </div>
      <div class="relative h-16 min-w-[40px] flex-1">
        <div
          class="absolute left-0 right-0 top-[31px] h-0.5 origin-left bg-white/15"
          style="animation:drawLine .9s ease both .2s;"
        ></div>
        <div
          class="absolute top-[28px] h-2 w-[30px] rounded-full"
          style="background:linear-gradient(90deg,transparent,var(--glow),transparent); filter:blur(.5px); animation:flow 2.2s linear infinite .9s;"
        ></div>
      </div>
      <div class="w-[184px] flex-none text-center">
        <div
          class="mx-auto flex h-16 w-16 items-center justify-center rounded-full border border-white/20 bg-white/5 font-serif text-[18px] font-semibold"
        >
          ML
        </div>
        <div class="mt-[13px] text-[14px] font-semibold">Mara Lindqvist</div>
        <div class="mt-1 font-mono text-[10px] uppercase tracking-[0.1em] text-[#8f8c80]">
          re-delegates
        </div>
      </div>
      <div class="relative h-16 min-w-[40px] flex-1">
        <div
          class="absolute left-0 right-0 top-[31px] h-0.5 origin-left bg-white/15"
          style="animation:drawLine .9s ease both .55s;"
        ></div>
        <div
          class="absolute top-[28px] h-2 w-[30px] rounded-full"
          style="background:linear-gradient(90deg,transparent,var(--glow),transparent); filter:blur(.5px); animation:flow 2.2s linear infinite 2s;"
        ></div>
      </div>
      <div class="w-[184px] flex-none text-center">
        <div
          class="mx-auto flex h-16 w-16 items-center justify-center rounded-full border-[1.5px] border-affirm-600 bg-[rgba(58,107,78,0.2)] font-serif text-[18px] font-semibold text-[#bfe0c9]"
          style="animation:ringPulse 3s ease-in-out infinite;"
        >
          OB
        </div>
        <div class="mt-[13px] text-[14px] font-semibold">Dr. Osei Boateng</div>
        <div
          class="mt-2 inline-flex items-center gap-1.5 rounded-full border border-affirm-600 bg-[rgba(58,107,78,0.22)] px-[11px] py-[5px] font-mono text-[10px] uppercase tracking-[0.1em] text-[#bfe0c9]"
        >
          ● Voted Yes
        </div>
      </div>
    </div>
  </div>
</section>

<!-- PRINCIPLES -->
<section class="mx-auto max-w-civic px-5 py-20 sm:px-10">
  <div class="mb-9 font-mono text-[11px] uppercase tracking-[0.2em] text-ink-400">
    What we hold to
  </div>
  <div
    class="grid grid-cols-1 gap-px overflow-hidden rounded border border-line bg-line sm:grid-cols-2"
  >
    <div class="bg-card px-8 py-[30px]">
      <h3 class="mb-2 font-serif text-[22px] font-semibold">Sovereignty</h3>
      <p class="font-serif text-[17px] leading-[1.5] text-ink-600">
        Users hold real authority over their own votes and delegations — never a recommendation
        engine deciding for them.
      </p>
    </div>
    <div class="bg-card px-8 py-[30px]">
      <h3 class="mb-2 font-serif text-[22px] font-semibold">Transparency</h3>
      <p class="font-serif text-[17px] leading-[1.5] text-ink-600">
        All tallies are publicly verifiable. Every state change is recorded in an append-only audit
        log.
      </p>
    </div>
    <div class="bg-card px-8 py-[30px]">
      <h3 class="mb-2 font-serif text-[22px] font-semibold">Reversibility</h3>
      <p class="font-serif text-[17px] leading-[1.5] text-ink-600">
        Votes can be changed while the window is open; delegations revoked at any time. Nothing you
        decide is a trap.
      </p>
    </div>
    <div class="bg-card px-8 py-[30px]">
      <h3 class="mb-2 font-serif text-[22px] font-semibold">Resistance to capture</h3>
      <p class="font-serif text-[17px] leading-[1.5] text-ink-600">
        Licensed AGPL-3.0 to keep the civic commons from corporate enclosure. The code itself
        supports self-governance.
      </p>
    </div>
  </div>
</section>

<!-- CTA -->
<section class="mx-auto max-w-civic px-5 pb-[90px] sm:px-10">
  <div class="rounded-[5px] border border-line bg-card px-12 py-14 text-center">
    <h2
      class="mx-auto max-w-[20ch] font-serif text-[clamp(28px,3.4vw,40px)] font-semibold leading-[1.1] tracking-[-0.015em]"
    >
      Collective decisions, made legible and reversible.
    </h2>
    <div class="mt-[30px] flex flex-wrap justify-center gap-3">
      <a
        href="/proposals"
        class="inline-flex items-center gap-2 rounded-[3px] bg-ink-900 px-6 py-[14px] text-[15px] font-semibold text-white"
      >
        Browse proposals <span class="font-mono">→</span>
      </a>
    </div>
  </div>
</section>
