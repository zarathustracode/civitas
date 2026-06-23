<script lang="ts">
  import { onMount } from 'svelte';
  import { invalidateAll } from '$app/navigation';
  import type { PageData, ActionData } from './$types';
  import type { VoteChoice, ProposalStatus, Stance } from '$lib/types/domain';
  import { initials } from '$lib/utils/text';
  import Markdown from '$lib/components/Markdown.svelte';
  import TallyDisplay from '$lib/components/TallyDisplay.svelte';
  import VoteInterface from '$lib/components/VoteInterface.svelte';
  import AuditTimeline from '$lib/components/AuditTimeline.svelte';
  import Banner from '$lib/components/Banner.svelte';
  import { friendlyMessage, ApiError } from '$lib/api/errors';

  let { data, form }: { data: PageData; form: ActionData } = $props();

  const voteError = $derived(
    form && 'code' in form && form.code
      ? friendlyMessage(new ApiError(form.code, form.code, 0))
      : null
  );
  const voteSuccess = $derived(form && 'voted' in form ? form.voted : null);

  const canVote = $derived.by(() => {
    if (!data.currentUser) return false;
    if (!data.currentUser.email_verified) return false;
    return data.proposal.status === 'voting';
  });
  const cantVoteReason = $derived.by(() => {
    if (!data.currentUser) return 'Sign in to vote on this proposal.';
    if (!data.currentUser.email_verified) return 'Verify your email before voting.';
    if (data.proposal.status === 'closed') return 'Voting has closed on this proposal.';
    if (data.proposal.status !== 'voting') return 'This proposal is not in the voting phase.';
    return undefined;
  });

  const choiceLabel: Record<VoteChoice, string> = { yes: 'Yes', no: 'No', abstain: 'Abstain' };

  // Live countdown to close.
  let now = $state(0);
  onMount(() => {
    now = Date.now();
    const t = setInterval(() => (now = Date.now()), 1000);
    return () => clearInterval(t);
  });
  const countdown = $derived.by(() => {
    if (data.proposal.status !== 'voting' || !data.proposal.voting_ends_at || !now) return null;
    let ms = Math.max(0, new Date(data.proposal.voting_ends_at).getTime() - now);
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

  const statusPill: Record<ProposalStatus, { label: string; color: string }> = {
    voting: { label: 'Voting open', color: 'affirm' },
    deliberation: { label: 'In deliberation', color: 'ochre' },
    closed: { label: 'Closed', color: 'ink' },
    draft: { label: 'Draft', color: 'ink' }
  };
  const pill = $derived(statusPill[data.proposal.status]);

  function relTime(iso: string): string {
    const ms = Date.now() - new Date(iso).getTime();
    const mins = Math.floor(ms / 60_000);
    if (mins < 60) return `${Math.max(mins, 1)}m ago`;
    const hrs = Math.floor(mins / 60);
    if (hrs < 24) return `${hrs}h ago`;
    return `${Math.floor(hrs / 24)}d ago`;
  }

  const trail = $derived(data.tally.your_trail);

  // Delegation-chain nodes, when the viewer's weight flows through a chain.
  const chainNodes = $derived.by(() => {
    if (!trail || trail.kind !== 'delegated') return null;
    const mids = trail.path.map((p) => ({ name: p.display_name, init: initials(p.display_name) }));
    const terminal = {
      name: trail.terminal.display_name,
      init: initials(trail.terminal.display_name)
    };
    return { mids, terminal, choice: trail.choice };
  });

  // Top-level, visible deliberation comments — a preview of the full thread.
  const previewComments = $derived(
    data.comments
      .filter((c) => c.parent_id === null && c.deleted_at === null && c.hidden_at === null)
      .slice(0, 3)
  );
  const stanceMeta: Record<Stance, { label: string; text: string; bg: string }> = {
    support: { label: 'Support', text: 'text-affirm-600', bg: 'bg-affirm-600' },
    oppose: { label: 'Oppose', text: 'text-oppose-600', bg: 'bg-oppose-600' },
    question: { label: 'Question', text: 'text-accent-600', bg: 'bg-accent-600' },
    neutral: { label: 'Neutral', text: 'text-ink-600', bg: 'bg-ink-400' }
  };

  const closedResult = $derived.by(() => {
    if (data.proposal.status !== 'closed') return null;
    const yes = parseFloat(data.tally.yes);
    const no = parseFloat(data.tally.no);
    if (yes + no + parseFloat(data.tally.abstain) === 0) return 'No votes were counted.';
    return yes > no ? 'Passed' : no > yes ? 'Failed' : 'Tied';
  });

  const votingWindow = $derived.by(() => {
    const s = data.proposal.voting_starts_at;
    const e = data.proposal.voting_ends_at;
    if (!s || !e) return null;
    const fmt = (iso: string) =>
      new Date(iso).toLocaleDateString('en-US', {
        month: 'short',
        day: 'numeric',
        year: 'numeric'
      });
    return `${fmt(s)} — ${fmt(e)}`;
  });
</script>

<svelte:head>
  <title>{data.proposal.title} — Civitas</title>
</svelte:head>

<!-- HERO -->
<section class="mx-auto max-w-civic px-5 pb-10 pt-16 sm:px-10">
  <p class="mb-6 font-mono text-[11px] uppercase tracking-[0.16em] text-ink-400">
    <a href="/proposals" class="hover:text-ink-600 hover:underline">Proposals</a> ›
  </p>
  <div class="mb-6 flex flex-wrap items-center gap-4">
    <span
      class="inline-flex items-center gap-2 rounded-full border px-3 py-1.5 font-mono text-[11px] uppercase tracking-[0.12em] {pill.color ===
      'affirm'
        ? 'border-affirm-600 text-affirm-600'
        : pill.color === 'ochre'
          ? 'border-ochre-600 text-ochre-600'
          : 'border-ink-400 text-ink-400'}"
    >
      <span
        class="h-[7px] w-[7px] rounded-full {pill.color === 'affirm'
          ? 'bg-affirm-600'
          : pill.color === 'ochre'
            ? 'bg-ochre-600'
            : 'bg-ink-400'}"
        style={data.proposal.status === 'voting' ? 'animation:blink 2.4s steps(1) infinite;' : ''}
      ></span>{pill.label}
    </span>
    <span class="font-mono text-[12px] uppercase tracking-[0.18em] text-ink-400">
      Proposal · {data.proposal.id.slice(0, 8)}
    </span>
    {#if countdown}
      <span class="ml-auto flex items-center gap-2.5">
        <span class="font-mono text-[10px] uppercase tracking-[0.16em] text-ink-400">closes in</span
        >
        <span class="font-mono text-[13px] font-medium tabular-nums tracking-[0.04em] text-ink-900">
          {countdown.days}d {countdown.h}:{countdown.m}:<span class="text-accent-600"
            >{countdown.s}</span
          >
        </span>
      </span>
    {/if}
  </div>

  <h1
    class="max-w-[16ch] font-serif text-[clamp(40px,5.4vw,66px)] font-semibold leading-[1.04] tracking-[-0.015em]"
  >
    {data.proposal.title}
  </h1>
  <p class="mt-[26px] max-w-[62ch] font-serif text-[22px] leading-[1.5] text-ink-600">
    {data.proposal.summary}
  </p>

  <div class="mt-10 flex flex-wrap gap-12 border-t border-line pt-6">
    <div>
      <div class="mb-1.5 font-mono text-[10px] uppercase tracking-[0.16em] text-ink-400">
        Deliberation
      </div>
      <div class="text-[15px] font-medium tabular-nums">
        {data.comments.length} comment{data.comments.length === 1 ? '' : 's'}
      </div>
    </div>
    {#if votingWindow}
      <div>
        <div class="mb-1.5 font-mono text-[10px] uppercase tracking-[0.16em] text-ink-400">
          Voting window
        </div>
        <div class="text-[15px] font-medium tabular-nums">{votingWindow}</div>
      </div>
    {/if}
    <div>
      <div class="mb-1.5 font-mono text-[10px] uppercase tracking-[0.16em] text-ink-400">
        Opened
      </div>
      <div class="text-[15px] font-medium tabular-nums">
        {new Date(data.proposal.created_at).toLocaleDateString('en-US', {
          month: 'short',
          day: 'numeric',
          year: 'numeric'
        })}
      </div>
    </div>
  </div>
</section>

{#if voteSuccess}
  <div class="mx-auto max-w-civic px-5 pb-2 sm:px-10">
    <Banner tone="success" title="Vote recorded">
      Your vote ({choiceLabel[voteSuccess]}) was recorded. You can change it until the voting window
      closes.
    </Banner>
  </div>
{/if}
{#if voteError}
  <div class="mx-auto max-w-civic px-5 pb-2 sm:px-10">
    <Banner tone="error" title="Could not record vote">{voteError}</Banner>
  </div>
{/if}

<!-- MAIN GRID -->
<section
  class="mx-auto grid max-w-civic items-start gap-14 px-5 pb-[72px] pt-6 sm:px-10 lg:grid-cols-[1.65fr_1fr]"
>
  <!-- DOCUMENT -->
  <article class="min-w-0">
    <div class="mb-3.5 font-mono text-[11px] uppercase tracking-[0.16em] text-accent-600">
      § The proposal
    </div>
    <Markdown source={data.proposal.body} />

    {#if closedResult}
      <div class="mt-9 rounded border border-line bg-card px-6 py-5">
        <div class="font-mono text-[10px] uppercase tracking-[0.16em] text-ink-400">Outcome</div>
        <div class="mt-1.5 font-serif text-[28px] font-semibold">{closedResult}</div>
      </div>
    {/if}
  </article>

  <!-- STICKY RAIL -->
  <aside class="flex flex-col gap-[18px] lg:sticky lg:top-[84px]">
    <!-- YOUR STANDING -->
    {#if data.currentUser && trail}
      <div class="rounded border border-line bg-card p-5">
        <div class="mb-3 font-mono text-[10px] uppercase tracking-[0.16em] text-ink-400">
          Your standing
        </div>
        <div class="flex items-start gap-3">
          <span
            class="flex h-[34px] w-[34px] flex-none items-center justify-center rounded-full font-serif text-[14px] font-semibold text-white {trail.kind ===
            'direct'
              ? 'bg-ink-900'
              : 'bg-accent-600'}"
          >
            {initials(data.currentUser.display_name)}
          </span>
          <div class="text-[14px] leading-[1.45]">
            {#if trail.kind === 'direct'}
              You voted <strong class="font-semibold">directly</strong>. Your weight counts as
              <strong class="font-semibold">{choiceLabel[trail.choice]}</strong> and no longer follows
              your delegation.
            {:else if trail.kind === 'delegated'}
              Your vote is <strong class="font-semibold">delegated</strong> on this topic. It
              follows your trust chain and currently counts as
              <strong class="font-semibold text-affirm-600">{choiceLabel[trail.choice]}</strong>.
            {:else}
              Your weight is not currently counted on this proposal.
            {/if}
          </div>
        </div>
      </div>
    {/if}

    <!-- CAST VOTE -->
    <div class="rounded border border-line bg-card p-5">
      <VoteInterface {canVote} notVotingReason={cantVoteReason} onSuccess={() => invalidateAll()} />
    </div>

    <!-- LIVE TALLY -->
    <div class="rounded border border-line bg-card p-5">
      <TallyDisplay tally={data.tally} live={data.proposal.status === 'voting'} />
    </div>
  </aside>
</section>

<!-- SIGNATURE BAND: how your vote travels -->
<section class="bg-band text-band-ink" style="--glow:#7d97ff;">
  <div class="mx-auto max-w-civic px-5 py-[72px] sm:px-10">
    <div class="mb-2 flex flex-wrap items-baseline justify-between gap-4">
      <div class="font-mono text-[11px] uppercase tracking-[0.2em] text-band-mute">
        The mechanism
      </div>
      <div class="font-mono text-[11px] uppercase tracking-[0.1em] text-band-mute">
        Transitive · cycle-checked · auditable
      </div>
    </div>
    <h2
      class="mb-2 font-serif text-[clamp(30px,4vw,46px)] font-medium leading-[1.08] tracking-[-0.01em]"
    >
      How your vote travels
    </h2>
    <p class="mb-12 max-w-[60ch] font-serif text-[19px] leading-[1.55] text-[#b9b6aa]">
      When you delegate, your weight flows along a chain of people you trust until it reaches
      someone who votes directly. Every link is shown — nothing is hidden, and a direct vote always
      wins.
    </p>

    {#if chainNodes}
      <!-- Real chain from the viewer's resolved trail. -->
      <div class="flex min-w-0 flex-wrap items-start gap-y-6">
        <div class="w-[160px] flex-none text-center">
          <div
            class="mx-auto flex h-16 w-16 items-center justify-center rounded-full border border-[rgba(125,151,255,0.5)] bg-[rgba(125,151,255,0.14)] font-serif text-[18px] font-semibold text-[#cdd6ff]"
          >
            You
          </div>
          <div class="mt-3 text-[14px] font-semibold">You</div>
          <div class="mt-1 font-mono text-[10px] uppercase tracking-[0.1em] text-[#8f8c80]">
            Delegating
          </div>
        </div>
        {#each chainNodes.mids as hop (hop.name)}
          <div class="relative h-16 min-w-[28px] flex-1">
            <div class="absolute left-0 right-0 top-[31px] h-0.5 bg-white/15"></div>
          </div>
          <div class="w-[160px] flex-none text-center">
            <div
              class="mx-auto flex h-16 w-16 items-center justify-center rounded-full border border-white/20 bg-white/5 font-serif text-[18px] font-semibold"
            >
              {hop.init}
            </div>
            <div class="mt-3 text-[14px] font-semibold">{hop.name}</div>
            <div class="mt-1 font-mono text-[10px] uppercase tracking-[0.1em] text-[#8f8c80]">
              re-delegates
            </div>
          </div>
        {/each}
        <div class="relative h-16 min-w-[28px] flex-1">
          <div class="absolute left-0 right-0 top-[31px] h-0.5 bg-white/15"></div>
        </div>
        <div class="w-[160px] flex-none text-center">
          <div
            class="mx-auto flex h-16 w-16 items-center justify-center rounded-full border-[1.5px] border-affirm-600 bg-[rgba(58,107,78,0.2)] font-serif text-[18px] font-semibold text-[#bfe0c9]"
            style="animation:ringPulse 3s ease-in-out infinite;"
          >
            {chainNodes.terminal.init}
          </div>
          <div class="mt-3 text-[14px] font-semibold">{chainNodes.terminal.name}</div>
          <div
            class="mt-2 inline-flex items-center gap-1.5 rounded-full border border-affirm-600 bg-[rgba(58,107,78,0.22)] px-[11px] py-[5px] font-mono text-[10px] uppercase tracking-[0.1em] text-[#bfe0c9]"
          >
            ● Voted {choiceLabel[chainNodes.choice]}
          </div>
        </div>
      </div>
    {:else if trail && trail.kind === 'direct'}
      <div
        class="inline-flex items-center gap-2.5 rounded-full border border-[rgba(176,73,47,0.6)] bg-[rgba(176,73,47,0.14)] px-4 py-2.5 font-mono text-[11px] uppercase tracking-[0.1em] text-[#e8a48f]"
      >
        ◆ Direct vote active — your weight no longer follows a delegation chain
      </div>
    {:else}
      <p class="font-mono text-[12px] leading-[1.7] tracking-[0.04em] text-[#8f8c80]">
        You are voting directly on this topic. Delegate it from your
        <a href="/delegations" class="text-[#cdd6ff] hover:underline">delegations</a> to let a trusted
        citizen carry your weight — and watch the chain appear here.
      </p>
    {/if}
  </div>
</section>

<!-- DELIBERATION -->
<section class="mx-auto max-w-civic px-5 py-16 sm:px-10">
  <div class="mb-7 flex items-baseline justify-between">
    <h2 class="font-serif text-[30px] font-semibold tracking-[-0.01em]">Deliberation</h2>
    <a
      href="/proposals/{data.proposal.id}/deliberate"
      class="font-mono text-[11px] uppercase tracking-[0.1em] text-accent-600 hover:underline"
    >
      View all {data.comments.length} →
    </a>
  </div>

  {#if previewComments.length === 0}
    <p
      class="rounded border border-dashed border-line px-6 py-6 font-serif text-[17px] text-ink-600"
    >
      No comments yet.
      <a href="/proposals/{data.proposal.id}/deliberate" class="text-accent-600 hover:underline"
        >Open the thread</a
      > to start the discussion.
    </p>
  {:else}
    <div class="overflow-hidden rounded border border-line">
      {#each previewComments as c, i (c.id)}
        {@const sm = stanceMeta[c.stance] ?? stanceMeta.neutral}
        <div class="bg-card px-6 py-[22px] {i > 0 ? 'border-t border-line' : ''}">
          <div class="mb-2.5 flex items-center gap-3">
            <span
              class="flex h-[30px] w-[30px] flex-none items-center justify-center rounded-full font-serif text-[12px] font-semibold text-white {sm.bg}"
            >
              {c.author_id.slice(0, 2).toUpperCase()}
            </span>
            <span class="font-mono text-[12px] text-ink-600">{c.author_id.slice(0, 8)}</span>
            <span
              class="inline-flex rounded-full px-2.5 py-1 font-mono text-[9px] uppercase tracking-[0.1em] {sm.text}"
              style="background:rgba(0,0,0,0.04);">{sm.label}</span
            >
            <span class="ml-auto font-mono text-[11px] text-ink-400">{relTime(c.created_at)}</span>
          </div>
          <p class="font-serif text-[17px] leading-[1.55] text-ink-900">{c.body}</p>
        </div>
      {/each}
    </div>
  {/if}
</section>

<!-- AUDIT -->
<section class="mx-auto max-w-civic px-5 pb-20 sm:px-10">
  <AuditTimeline entries={data.audit} />
</section>
