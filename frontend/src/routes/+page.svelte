<script lang="ts">
  import type { PageData } from './$types';
  let { data }: { data: PageData } = $props();
</script>

<svelte:head>
  <title>Civitas — direct democracy with delegation</title>
</svelte:head>

<section class="prose-civic">
  <h1>Civitas</h1>
  <p class="text-lg">
    A platform for direct democracy with optional per-topic delegation. Verified citizens vote
    directly on proposals or, when they prefer, delegate their vote on a topic to someone they
    trust.
  </p>

  <h2>How it works</h2>
  <ol class="ml-6 list-decimal space-y-2">
    <li>
      Register and verify your email. You become eligible to vote and to participate in
      deliberation.
    </li>
    <li>
      Browse <a href="/proposals">active proposals</a>. Read the summary, the full body, and the
      deliberation thread.
    </li>
    <li>
      Cast a direct vote — yes, no, or abstain — or delegate your vote on a topic to another
      verified citizen.
    </li>
    <li>
      Tallies are computed openly: every weight that was counted is reported with the chain that
      carried it.
    </li>
  </ol>

  <h2>Why this exists</h2>
  <p>
    Most political software is built either to capture attention or to entrench incumbents. Civitas
    is built to do neither. It is licensed AGPL-3.0 specifically to prevent corporate enclosure of a
    civic commons. The political philosophy is documented in the repository.
  </p>

  <p class="rounded-md bg-ink-100 px-4 py-3 text-sm text-ink-800">
    <strong>Status:</strong> early development under founder control. The project is committed to
    migrating its own governance onto the platform — see
    <a href="https://github.com/zarathustracode/civitas/blob/main/GOVERNANCE.md">GOVERNANCE.md</a>.
  </p>

  <div class="flex flex-wrap gap-3 pt-2">
    <a
      href="/proposals"
      class="inline-flex items-center rounded-md bg-accent-600 px-4 py-2 text-white hover:bg-accent-700"
    >
      Browse proposals
    </a>
    {#if !data.currentUser}
      <a
        href="/auth/register"
        class="inline-flex items-center rounded-md border border-ink-200 bg-white px-4 py-2 text-ink-800 hover:bg-ink-50"
      >
        Create an account
      </a>
    {/if}
  </div>
</section>
