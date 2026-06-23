<script lang="ts">
  import { enhance } from '$app/forms';
  import type { VoteChoice } from '$lib/types/domain';

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

  const choices: VoteChoice[] = ['yes', 'no', 'abstain'];
  const label: Record<VoteChoice, string> = { yes: 'Yes', no: 'No', abstain: 'Abstain' };
  const sub: Record<VoteChoice, string> = { yes: 'affirm', no: 'oppose', abstain: 'present' };
  const summary: Record<VoteChoice, string> = {
    yes: 'You affirm this proposal.',
    no: 'You oppose this proposal.',
    abstain: 'You are present, registering no preference.'
  };
  const hoverByChoice: Record<VoteChoice, string> = {
    yes: 'hover:border-affirm-600 hover:bg-affirm-50',
    no: 'hover:border-oppose-600 hover:bg-oppose-50',
    abstain: 'hover:border-neutral-600 hover:bg-[#f6f5f1]'
  };
</script>

<section aria-labelledby="vote-heading">
  <div
    id="vote-heading"
    class="mb-3.5 font-mono text-[10px] uppercase tracking-[0.16em] text-ink-400"
  >
    Cast your vote
  </div>

  {#if !canVote}
    <p class="rounded-[3px] bg-ink-100 px-4 py-3 text-[13px] leading-[1.45] text-ink-600">
      {notVotingReason ?? 'You cannot vote on this proposal right now.'}
    </p>
  {:else if pending === null}
    <div class="flex flex-col gap-2">
      {#each choices as c (c)}
        <button
          type="button"
          onclick={() => (pending = c)}
          class="flex cursor-pointer items-center justify-between rounded-[3px] border border-line bg-white px-4 py-[13px] text-[15px] font-semibold text-ink-900 transition-all hover:-translate-y-px {hoverByChoice[
            c
          ]}"
        >
          {label[c]}<span class="font-mono text-[11px] font-normal text-ink-400">{sub[c]}</span>
        </button>
      {/each}
    </div>
    <p class="mt-3 text-[12px] leading-[1.45] text-ink-400">
      A direct vote overrides your delegation. You can change it until voting closes.
    </p>
  {:else}
    <form
      method="POST"
      action="?/vote"
      class="rounded-[3px] border border-accent-600 bg-accent-50 p-4"
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
      <p class="text-[14px] leading-[1.5]">
        You are voting <strong class="font-bold">{label[pending]}</strong>. {summary[pending]}
      </p>
      <input type="hidden" name="choice" value={pending} />
      <div class="mt-3.5 flex gap-2">
        <button
          type="submit"
          disabled={submitting}
          class="flex-1 rounded-[3px] bg-accent-600 px-3.5 py-[11px] text-[14px] font-semibold text-white transition-colors hover:bg-accent-700 disabled:opacity-60"
        >
          {submitting ? 'Confirming…' : 'Confirm vote'}
        </button>
        <button
          type="button"
          disabled={submitting}
          onclick={() => (pending = null)}
          class="rounded-[3px] border border-line bg-white px-4 py-[11px] text-[14px] font-medium text-ink-600 disabled:opacity-60"
        >
          Cancel
        </button>
      </div>
    </form>
  {/if}
</section>
