<script lang="ts">
  import type { PageData } from './$types';
  let { data }: { data: PageData } = $props();
</script>

<svelte:head>
  <title>Topics — Civitas</title>
</svelte:head>

<section class="prose-civic">
  <h1>Topics</h1>
  <p>
    Topics are the unit of delegation. You can delegate your vote on different topics to different
    people you trust.
  </p>

  {#if data.topics.length === 0}
    <p class="rounded-md bg-ink-100 px-4 py-3 text-sm text-ink-800">
      No topics yet. {#if data.currentUser}
        Verified accounts can create one when proposals call for it.
      {/if}
    </p>
  {:else}
    <ul class="space-y-2">
      {#each data.topics as topic (topic.id)}
        <li>
          <a
            href="/topics/{topic.slug}"
            class="block rounded-lg border border-ink-200 bg-white p-3 hover:border-accent-500"
          >
            <p class="font-semibold">{topic.name}</p>
            {#if topic.description}
              <p class="text-sm text-ink-600">{topic.description}</p>
            {/if}
          </a>
        </li>
      {/each}
    </ul>
  {/if}
</section>
