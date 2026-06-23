<script lang="ts">
  import { enhance } from '$app/forms';
  import type { PageData, ActionData } from './$types';
  import UserSearchField from '$lib/components/UserSearchField.svelte';
  import Banner from '$lib/components/Banner.svelte';
  import { friendlyMessage, ApiError } from '$lib/api/errors';
  import { initials } from '$lib/utils/text';

  let { data, form }: { data: PageData; form: ActionData } = $props();

  const errorMessage = $derived(
    form && 'code' in form && form.code
      ? friendlyMessage(new ApiError(form.code, form.code, 0))
      : null
  );

  let submitting = $state(false);
  let selTopic = $state('');
  let delegateId = $state('');
  const valid = $derived(selTopic.length > 0 && delegateId.length > 0);

  const topicName = (id: string) => data.topics.find((t) => t.id === id)?.name ?? id;
  const usedTopicIds = $derived(new Set(data.mine.map((d) => d.topic_id)));
  const availableTopics = $derived(data.topics.filter((t) => !usedTopicIds.has(t.id)));
</script>

<svelte:head>
  <title>Delegations — Civitas</title>
</svelte:head>

<!-- HEADER -->
<section class="mx-auto max-w-civic px-5 pb-7 pt-14 sm:px-10">
  <div
    class="mb-3.5 font-mono text-[11px] uppercase tracking-[0.2em] text-ink-400"
    style="animation:fadeUp .6s both;"
  >
    Your trust network
  </div>
  <h1
    class="font-serif text-[clamp(40px,5.4vw,60px)] font-semibold leading-[1.04] tracking-[-0.015em]"
    style="animation:fadeUp .6s both .08s;"
  >
    Delegations
  </h1>
  <p
    class="mt-[18px] max-w-[60ch] font-serif text-[20px] leading-[1.5] text-ink-600"
    style="animation:fadeUp .6s both .14s;"
  >
    For each topic you may delegate your vote to one person you trust. Delegation is transitive —
    they may pass it onward — but a direct vote always overrides it, and you can revoke at any
    moment.
  </p>

  <!-- STAT LEDGER -->
  <div
    class="mt-10 grid grid-cols-1 gap-px overflow-hidden rounded border border-line bg-line sm:grid-cols-3"
    style="animation:fadeUp .6s both .2s;"
  >
    <div class="bg-card px-6 py-[22px]">
      <div class="font-mono text-[34px] font-medium tabular-nums tracking-[-0.02em]">
        {data.mine.length}
      </div>
      <div class="mt-1.5 font-mono text-[10px] uppercase tracking-[0.12em] text-ink-400">
        Topics delegated
      </div>
    </div>
    <div class="bg-card px-6 py-[22px]">
      <div class="font-mono text-[34px] font-medium tabular-nums tracking-[-0.02em]">
        {Math.max(data.topics.length - data.mine.length, 0)}
      </div>
      <div class="mt-1.5 font-mono text-[10px] uppercase tracking-[0.12em] text-ink-400">
        Topics you vote directly
      </div>
    </div>
    <div class="bg-card px-6 py-[22px]">
      <div class="font-mono text-[34px] font-medium tabular-nums tracking-[-0.02em]">
        {data.topics.length}
      </div>
      <div class="mt-1.5 font-mono text-[10px] uppercase tracking-[0.12em] text-ink-400">
        Topics in total
      </div>
    </div>
  </div>
</section>

{#if errorMessage}
  <div class="mx-auto max-w-civic px-5 pb-2 sm:px-10">
    <Banner tone="error" title="Action failed">{errorMessage}</Banner>
  </div>
{/if}

<!-- MAIN GRID -->
<section
  class="mx-auto grid max-w-civic items-start gap-12 px-5 pb-20 pt-[18px] sm:px-10 lg:grid-cols-[1.5fr_1fr]"
>
  <!-- ACTIVE DELEGATIONS -->
  <div class="min-w-0">
    <h2 class="mb-[18px] font-serif text-[24px] font-semibold tracking-[-0.01em]">
      Active delegations
    </h2>

    {#if data.mine.length === 0}
      <div
        class="rounded border border-dashed border-line px-7 py-7 font-serif text-[18px] text-ink-600"
      >
        You have no active delegations. Your vote is direct on every topic.
      </div>
    {:else}
      <div class="flex flex-col gap-3">
        {#each data.mine as d (d.id)}
          <div
            class="rounded border border-line bg-card px-[22px] py-5"
            style="animation:popIn .3s ease both;"
          >
            <div class="flex items-center justify-between gap-4">
              <span class="font-mono text-[10px] uppercase tracking-[0.12em] text-accent-600">
                {topicName(d.topic_id)}
              </span>
              <form method="POST" action="?/revoke" use:enhance>
                <input type="hidden" name="id" value={d.id} />
                <button
                  type="submit"
                  class="cursor-pointer rounded-full border border-line bg-transparent px-3 py-1.5 font-mono text-[10px] uppercase tracking-[0.08em] text-ink-600 transition-colors hover:border-oppose-600 hover:text-oppose-600"
                >
                  Revoke
                </button>
              </form>
            </div>

            <!-- MINI CHAIN -->
            <div class="mt-[18px] flex items-center gap-0">
              <span
                class="flex h-[38px] w-[38px] flex-none items-center justify-center rounded-full border border-[rgba(43,58,140,0.4)] bg-[rgba(43,58,140,0.12)] font-serif text-[13px] font-semibold text-accent-600"
              >
                You
              </span>
              <div
                class="mx-3 h-0.5 min-w-[24px] flex-1"
                style="background:linear-gradient(90deg,var(--accent),var(--ink3));"
              ></div>
              <div class="flex flex-none items-center gap-2.5">
                <span
                  class="flex h-[38px] w-[38px] flex-none items-center justify-center rounded-full bg-ink-900 font-serif text-[13px] font-semibold text-white"
                >
                  {initials(d.delegate_display_name ?? '?')}
                </span>
                <div>
                  <div class="text-[15px] font-semibold leading-[1.2]">
                    {#if d.delegate_display_name}
                      {d.delegate_display_name}
                    {:else}
                      <code class="font-mono text-[12px]">{d.delegate_id.slice(0, 12)}</code>
                    {/if}
                  </div>
                  <div
                    class="mt-[3px] font-mono text-[10px] uppercase tracking-[0.06em] text-ink-400"
                  >
                    carries your weight
                  </div>
                </div>
              </div>
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </div>

  <!-- SIDE: CREATE + EXPLAINER -->
  <aside class="flex flex-col gap-[18px]">
    <!-- CREATE FORM -->
    <div class="rounded border border-line bg-card p-[22px]">
      <h2 class="mb-4 font-serif text-[20px] font-semibold">Delegate on a new topic</h2>

      {#if availableTopics.length === 0}
        <p class="font-serif text-[16px] text-ink-600">
          You have delegated on every available topic. Revoke one to redirect it.
        </p>
      {:else}
        <form
          method="POST"
          action="?/create"
          class="flex flex-col gap-3.5"
          use:enhance={() => {
            submitting = true;
            return async ({ update }) => {
              await update();
              submitting = false;
              selTopic = '';
              delegateId = '';
            };
          }}
        >
          <label class="flex flex-col gap-1.5">
            <span class="font-mono text-[10px] uppercase tracking-[0.12em] text-ink-400">Topic</span
            >
            <select
              name="topic_id"
              bind:value={selTopic}
              required
              class="appearance-none rounded-[3px] border border-line bg-white px-3.5 py-[11px] text-[15px] text-ink-900 focus:border-accent-500 focus:outline-2 focus:outline-offset-2 focus:outline-accent-500"
            >
              <option value="">Select a topic…</option>
              {#each availableTopics as t (t.id)}
                <option value={t.id}>{t.name}</option>
              {/each}
            </select>
          </label>

          <UserSearchField
            name="delegate_id"
            label="Delegate"
            bind:selectedId={delegateId}
            hint="Search by name or email. The directory is restricted to verified citizens. Cycles are rejected at creation."
          />

          <button
            type="submit"
            disabled={!valid || submitting}
            class="rounded-[3px] px-4 py-3 text-[14px] font-semibold text-white transition-colors {valid &&
            !submitting
              ? 'bg-accent-600 hover:bg-accent-700'
              : 'cursor-not-allowed bg-ink-400'}"
          >
            {submitting ? 'Creating…' : 'Create delegation'}
          </button>
        </form>
      {/if}
    </div>

    <!-- EXPLAINER -->
    <div class="rounded bg-band p-[22px] text-band-ink">
      <div class="mb-3.5 font-mono text-[10px] uppercase tracking-[0.16em] text-band-mute">
        When others trust you
      </div>
      <p class="font-serif text-[17px] leading-[1.55] text-[#cfccc1]">
        Weight delegated to you flows through your direct votes on those topics — your delegates'
        choices follow yours.
      </p>
      <p class="mt-3.5 font-mono text-[11px] leading-[1.7] tracking-[0.04em] text-[#8f8c80]">
        Vote thoughtfully on topics where others rely on you. Every delegation and every vote it
        carries is recorded in the audit log.
      </p>
    </div>
  </aside>
</section>
