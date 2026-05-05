<script lang="ts">
  import { enhance } from '$app/forms';
  import type { VoteChoice } from '$lib/types/domain';
  import Button from './Button.svelte';

  let {
    canVote,
    notVotingReason,
    onSuccess
  }: {
    canVote: boolean;
    notVotingReason?: string;
    onSuccess?: () => void;
  } = $props();

  let pending = $state<VoteChoice | null>(null);
  let submitting = $state(false);

  const labelByChoice: Record<VoteChoice, string> = {
    yes: 'Yes',
    no: 'No',
    abstain: 'Abstain'
  };
  const summaryByChoice: Record<VoteChoice, string> = {
    yes: 'You agree with this proposal.',
    no: 'You disagree with this proposal.',
    abstain: 'You are present and registering no preference.'
  };
</script>

<section aria-labelledby="vote-heading" class="space-y-3">
  <h3 id="vote-heading" class="text-lg font-semibold">Cast your vote</h3>

  {#if !canVote}
    <p class="rounded-md bg-ink-100 px-4 py-3 text-sm text-ink-800">
      {notVotingReason ?? 'You cannot vote on this proposal right now.'}
    </p>
  {:else if pending === null}
    <div class="grid gap-2 sm:grid-cols-3">
      <Button variant="primary" onclick={() => (pending = 'yes')}>Yes</Button>
      <Button variant="danger" onclick={() => (pending = 'no')}>No</Button>
      <Button variant="secondary" onclick={() => (pending = 'abstain')}>Abstain</Button>
    </div>
    <p class="text-xs text-ink-600">You can change your vote until the voting window closes.</p>
  {:else}
    <form
      method="POST"
      action="?/vote"
      class="rounded-md border border-accent-500 bg-accent-50 p-4"
      use:enhance={() => {
        submitting = true;
        return async ({ update }) => {
          await update();
          submitting = false;
          pending = null;
          onSuccess?.();
        };
      }}
    >
      <p class="text-sm">
        You are voting <strong>{labelByChoice[pending]}</strong>. {summaryByChoice[pending]}
      </p>
      <input type="hidden" name="choice" value={pending} />
      <div class="mt-3 flex flex-wrap gap-2">
        <Button type="submit" variant="primary" loading={submitting}>Confirm vote</Button>
        <Button variant="ghost" disabled={submitting} onclick={() => (pending = null)}>
          Cancel
        </Button>
      </div>
    </form>
  {/if}
</section>
