<script lang="ts">
  import { enhance } from '$app/forms';
  import type { PageData, ActionData } from './$types';
  import Button from '$lib/components/Button.svelte';
  import TextField from '$lib/components/TextField.svelte';
  import Banner from '$lib/components/Banner.svelte';
  import { friendlyMessage, ApiError } from '$lib/api/errors';

  let { data, form }: { data: PageData; form: ActionData } = $props();

  const errorMessage = $derived(
    form && 'code' in form && form.code
      ? friendlyMessage(new ApiError(form.code, form.code, 0))
      : null
  );

  let submitting = $state(false);

  function topicNameFor(id: string): string {
    return data.topics.find((t) => t.id === id)?.name ?? id;
  }
</script>

<svelte:head>
  <title>Delegations — Civitas</title>
</svelte:head>

<section class="space-y-6">
  <header class="prose-civic">
    <h1>Your delegations</h1>
    <p>
      For each topic you can delegate your vote to a single trusted person. Direct votes always
      override delegation: voting on a specific proposal does not affect your topic-level delegation
      for future proposals.
    </p>
  </header>

  {#if errorMessage}
    <Banner tone="error" title="Action failed">{errorMessage}</Banner>
  {/if}

  <div class="space-y-3">
    <h2 class="text-lg font-semibold">Active delegations</h2>
    {#if data.mine.length === 0}
      <p class="rounded-md bg-ink-100 px-4 py-3 text-sm text-ink-800">
        You have no active delegations. Your votes are direct on every topic.
      </p>
    {:else}
      <ul class="space-y-2">
        {#each data.mine as d (d.id)}
          <li
            class="flex flex-wrap items-center justify-between gap-3 rounded-lg border border-ink-200 bg-white p-3"
          >
            <div>
              <p class="font-medium">{topicNameFor(d.topic_id)}</p>
              <p class="text-sm text-ink-600">
                delegated to <code class="font-mono text-xs">{d.delegate_id}</code>
              </p>
            </div>
            <form method="POST" action="?/revoke" use:enhance>
              <input type="hidden" name="id" value={d.id} />
              <Button type="submit" variant="secondary" size="sm">Revoke</Button>
            </form>
          </li>
        {/each}
      </ul>
    {/if}
  </div>

  <div class="space-y-3 rounded-lg border border-ink-200 bg-white p-4">
    <h2 class="text-lg font-semibold">Delegate on a new topic</h2>
    <form
      method="POST"
      action="?/create"
      class="flex flex-col gap-3"
      use:enhance={() => {
        submitting = true;
        return async ({ update }) => {
          await update();
          submitting = false;
        };
      }}
    >
      <label class="flex flex-col gap-1 text-sm">
        <span class="font-medium">Topic</span>
        <select
          name="topic_id"
          required
          class="rounded-md border border-ink-200 px-3 py-2 focus:outline-2 focus:outline-offset-2 focus:outline-accent-500"
        >
          <option value="">Select a topic…</option>
          {#each data.topics as t (t.id)}
            <option value={t.id}>{t.name}</option>
          {/each}
        </select>
      </label>
      <TextField
        name="delegate_id"
        label="Delegate (user ID)"
        required
        hint="Enter the UUID of the user you want to delegate to. v0.1 does not yet have a search interface."
      />
      <div>
        <Button type="submit" loading={submitting}>Create delegation</Button>
      </div>
    </form>
  </div>
</section>
