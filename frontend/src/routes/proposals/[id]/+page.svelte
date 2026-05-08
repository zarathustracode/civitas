<script lang="ts">
  import { invalidateAll } from '$app/navigation';
  import type { PageData, ActionData } from './$types';
  import StatusBadge from '$lib/components/StatusBadge.svelte';
  import TallyDisplay from '$lib/components/TallyDisplay.svelte';
  import VoteInterface from '$lib/components/VoteInterface.svelte';
  import YourTrail from '$lib/components/YourTrail.svelte';
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
    if (!data.currentUser) return 'You need to log in to vote.';
    if (!data.currentUser.email_verified) return 'Verify your email before voting.';
    if (data.proposal.status === 'closed') return 'Voting has closed on this proposal.';
    if (data.proposal.status !== 'voting') return 'This proposal is not in the voting phase.';
    return undefined;
  });
</script>

<svelte:head>
  <title>{data.proposal.title} — Civitas</title>
</svelte:head>

<article class="space-y-6">
  <header class="space-y-2">
    <p class="text-sm text-ink-600">
      <a href="/proposals" class="hover:underline">Proposals</a> ›
    </p>
    <div class="flex flex-wrap items-start justify-between gap-3">
      <h1 class="text-3xl font-semibold">{data.proposal.title}</h1>
      <StatusBadge status={data.proposal.status} />
    </div>
    <p class="text-ink-700">{data.proposal.summary}</p>
  </header>

  {#if voteSuccess}
    <Banner tone="success" title="Vote recorded">
      Your vote ({voteSuccess}) was recorded. You can change it until the voting window closes.
    </Banner>
  {/if}
  {#if voteError}
    <Banner tone="error" title="Could not record vote">{voteError}</Banner>
  {/if}

  <section class="prose-civic">
    <h2 class="sr-only">Body</h2>
    <p class="whitespace-pre-line">{data.proposal.body}</p>
  </section>

  <div class="grid gap-4 md:grid-cols-2">
    <div class="rounded-lg border border-ink-200 bg-white p-4">
      <VoteInterface {canVote} notVotingReason={cantVoteReason} onSuccess={() => invalidateAll()} />
    </div>
    <div class="rounded-lg border border-ink-200 bg-white p-4">
      <TallyDisplay tally={data.tally} />
    </div>
  </div>

  <YourTrail trail={data.tally.your_trail} />

  <div>
    <a
      href="/proposals/{data.proposal.id}/deliberate"
      class="inline-flex items-center text-accent-600 hover:underline"
    >
      Deliberation thread ({data.comments.length}) →
    </a>
  </div>
</article>
