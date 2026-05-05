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
</script>

<svelte:head>
  <title>Deliberate — {data.proposal.title}</title>
</svelte:head>

<section class="space-y-6">
  <header class="space-y-2">
    <p class="text-sm text-ink-600">
      <a href="/proposals" class="hover:underline">Proposals</a> ›
      <a href="/proposals/{data.proposal.id}" class="hover:underline">{data.proposal.title}</a> ›
    </p>
    <div class="flex flex-wrap items-start justify-between gap-3">
      <h1 class="text-2xl font-semibold">Deliberation</h1>
      <StatusBadge status={data.proposal.status} />
    </div>
    <p class="text-sm text-ink-600">
      {stanceCounts.support} support · {stanceCounts.oppose} oppose · {stanceCounts.neutral}
      neutral · {stanceCounts.question} questions
    </p>
  </header>

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
      class="space-y-3 rounded-lg border border-ink-200 bg-white p-4"
      use:enhance={() => {
        submitting = true;
        return async ({ update }) => {
          await update();
          submitting = false;
        };
      }}
    >
      <h2 class="text-lg font-semibold">Add to the deliberation</h2>
      <fieldset class="flex flex-wrap gap-3 text-sm">
        <legend class="sr-only">Stance</legend>
        {#each ['support', 'oppose', 'neutral', 'question'] as stance (stance)}
          <label class="inline-flex items-center gap-1">
            <input type="radio" name="stance" value={stance} required />
            <span class="capitalize">{stance}</span>
          </label>
        {/each}
      </fieldset>
      <TextField name="body" label="Your comment" multiline required rows={4} />
      <Button type="submit" loading={submitting}>Post comment</Button>
    </form>
  {:else if !data.currentUser}
    <p class="rounded-md bg-ink-100 px-4 py-3 text-sm text-ink-800">
      <a href="/auth/login" class="text-accent-600 underline">Log in</a> to participate in deliberation.
    </p>
  {:else if data.proposal.status === 'closed'}
    <p class="rounded-md bg-ink-100 px-4 py-3 text-sm text-ink-800">
      This proposal is closed; the thread is now read-only.
    </p>
  {:else if data.proposal.status === 'draft'}
    <p class="rounded-md bg-ink-100 px-4 py-3 text-sm text-ink-800">
      Deliberation has not opened on this proposal yet.
    </p>
  {:else}
    <p class="rounded-md bg-ink-100 px-4 py-3 text-sm text-ink-800">
      Verify your email before posting.
    </p>
  {/if}

  {#if data.comments.length === 0}
    <p class="text-sm text-ink-600">No comments yet.</p>
  {:else}
    <DeliberationThread comments={data.comments} />
  {/if}
</section>
