<script lang="ts">
  import { enhance } from '$app/forms';
  import type { PageData, ActionData } from './$types';
  import DeliberationThread from '$lib/components/DeliberationThread.svelte';
  import Button from '$lib/components/Button.svelte';
  import TextField from '$lib/components/TextField.svelte';
  import Banner from '$lib/components/Banner.svelte';
  import StatusBadge from '$lib/components/StatusBadge.svelte';
  import { friendlyMessage, ApiError } from '$lib/api/errors';

  let { data, form }: { data: PageData; form: ActionData } = $props();
  let submitting = $state(false);

  const errorMessage = $derived(
    form && 'code' in form && form.code
      ? friendlyMessage(new ApiError(form.code, form.code, 0))
      : null
  );
  const justPosted = $derived(form && 'posted' in form && form.posted);

  const stanceCounts = $derived.by(() => {
    const counts = { support: 0, oppose: 0, neutral: 0, question: 0 };
    for (const c of data.comments) {
      if (!c.deleted_at && !c.hidden_at) counts[c.stance]++;
    }
    return counts;
  });

  const canPost = $derived.by(() => {
    if (!data.currentUser) return false;
    if (!data.currentUser.email_verified) return false;
    return data.proposal.status === 'deliberation' || data.proposal.status === 'voting';
  });

  const stances = ['support', 'oppose', 'neutral', 'question'];
</script>

<svelte:head>
  <title>Deliberate — {data.proposal.title}</title>
</svelte:head>

<section class="mx-auto max-w-[860px] px-5 pb-20 pt-14 sm:px-10">
  <p class="mb-6 font-mono text-[11px] uppercase tracking-[0.16em] text-ink-400">
    <a href="/proposals" class="hover:text-ink-600 hover:underline">Proposals</a> ›
    <a href="/proposals/{data.proposal.id}" class="hover:text-ink-600 hover:underline"
      >{data.proposal.title}</a
    > ›
  </p>
  <div class="flex flex-wrap items-center justify-between gap-3">
    <h1
      class="font-serif text-[clamp(32px,4.4vw,44px)] font-semibold leading-[1.05] tracking-[-0.015em]"
    >
      Deliberation
    </h1>
    <StatusBadge status={data.proposal.status} />
  </div>
  <p class="mt-3 font-mono text-[12px] tracking-[0.04em] text-ink-400">
    {stanceCounts.support} support · {stanceCounts.oppose} oppose · {stanceCounts.neutral} neutral ·
    {stanceCounts.question} questions
  </p>

  <div class="mt-7 space-y-5">
    {#if justPosted}
      <Banner tone="success" title="Comment posted">Your comment is in the thread.</Banner>
    {/if}
    {#if errorMessage}
      <Banner tone="error" title="Could not post">{errorMessage}</Banner>
    {/if}

    {#if canPost}
      <form
        method="POST"
        action="?/post"
        class="space-y-4 rounded border border-line bg-card p-5"
        use:enhance={() => {
          submitting = true;
          return async ({ update }) => {
            await update();
            submitting = false;
          };
        }}
      >
        <h2 class="font-serif text-[20px] font-semibold">Add to the deliberation</h2>
        <fieldset class="flex flex-wrap gap-2">
          <legend class="sr-only">Stance</legend>
          {#each stances as stance (stance)}
            <label
              class="inline-flex cursor-pointer items-center gap-2 rounded-full border border-line bg-white px-3.5 py-2 font-mono text-[11px] uppercase tracking-[0.08em] text-ink-600 transition-colors has-[:checked]:border-accent-600 has-[:checked]:text-accent-600"
            >
              <input type="radio" name="stance" value={stance} required class="accent-accent-600" />
              {stance}
            </label>
          {/each}
        </fieldset>
        <TextField name="body" label="Your comment" multiline required rows={4} />
        <Button type="submit" loading={submitting}>Post comment</Button>
      </form>
    {:else if !data.currentUser}
      <p class="rounded-[3px] bg-ink-100 px-4 py-3 text-[14px] text-ink-600">
        <a href="/auth/login" class="text-accent-600 underline">Log in</a> to participate in deliberation.
      </p>
    {:else if data.proposal.status === 'closed'}
      <p class="rounded-[3px] bg-ink-100 px-4 py-3 text-[14px] text-ink-600">
        This proposal is closed; the thread is now read-only.
      </p>
    {:else if data.proposal.status === 'draft'}
      <p class="rounded-[3px] bg-ink-100 px-4 py-3 text-[14px] text-ink-600">
        Deliberation has not opened on this proposal yet.
      </p>
    {:else}
      <p class="rounded-[3px] bg-ink-100 px-4 py-3 text-[14px] text-ink-600">
        Verify your email before posting.
      </p>
    {/if}

    {#if data.comments.length === 0}
      <p class="font-serif text-[16px] text-ink-600">No comments yet.</p>
    {:else}
      <DeliberationThread comments={data.comments} />
    {/if}
  </div>
</section>
