<script lang="ts">
  import type { UserTrail } from '$lib/types/domain';

  let { trail }: { trail: UserTrail | null } = $props();

  const choiceLabel: Record<'yes' | 'no' | 'abstain', string> = {
    yes: 'Yes',
    no: 'No',
    abstain: 'Abstain'
  };
  const reasonLabel: Record<string, string> = {
    no_direct_vote_in_chain: 'Your delegation chain ends without a direct vote on this proposal.',
    depth_exceeded: 'Your delegation chain is too deep to resolve.'
  };
</script>

{#if trail}
  <section
    aria-labelledby="your-trail-heading"
    class="space-y-2 rounded-md border border-ink-200 bg-ink-50 p-3 text-sm"
  >
    <h3 id="your-trail-heading" class="font-semibold">How your weight flows</h3>
    {#if trail.kind === 'direct'}
      <p>
        You voted <strong>{choiceLabel[trail.choice]}</strong> directly. Your vote counts as cast.
      </p>
    {:else if trail.kind === 'delegated'}
      <p>
        You did not vote directly. Your weight flows through
        {#each trail.path as hop (hop.id)}
          <strong>{hop.display_name}</strong> →
        {/each}
        <strong>{trail.terminal.display_name}</strong>, who voted
        <strong>{choiceLabel[trail.choice]}</strong>.
      </p>
    {:else}
      <p>{reasonLabel[trail.reason] ?? `Your weight is not counted (${trail.reason}).`}</p>
    {/if}
  </section>
{/if}
