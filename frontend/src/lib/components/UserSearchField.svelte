<script lang="ts">
  import { searchUsers } from '$lib/api/users';
  import type { NamedUser } from '$lib/types/domain';

  let {
    name,
    label,
    selectedId = $bindable(''),
    hint
  }: {
    name: string;
    label: string;
    selectedId?: string;
    hint?: string;
  } = $props();

  let query = $state('');
  let results = $state<NamedUser[]>([]);
  let chosen = $state<NamedUser | null>(null);
  let loading = $state(false);
  let activeRequest: AbortController | null = null;

  async function runSearch(q: string) {
    if (q.trim().length < 2) {
      results = [];
      return;
    }
    activeRequest?.abort();
    const ctrl = new AbortController();
    activeRequest = ctrl;
    loading = true;
    try {
      const res = await searchUsers(q);
      if (!ctrl.signal.aborted) results = res;
    } finally {
      if (activeRequest === ctrl) activeRequest = null;
      loading = false;
    }
  }

  function pick(u: NamedUser) {
    chosen = u;
    selectedId = u.id;
    query = u.display_name;
    results = [];
  }

  function clear() {
    chosen = null;
    selectedId = '';
    query = '';
    results = [];
  }

  let debounce: ReturnType<typeof setTimeout> | null = null;
  function onInput(value: string) {
    query = value;
    if (chosen && value !== chosen.display_name) {
      chosen = null;
      selectedId = '';
    }
    if (debounce) clearTimeout(debounce);
    debounce = setTimeout(() => runSearch(value), 200);
  }

  const inputClass =
    'w-full rounded-[3px] border border-line bg-white px-3.5 py-[11px] text-[15px] text-ink-900 placeholder:text-ink-400 focus:border-accent-500 focus:outline-2 focus:outline-offset-2 focus:outline-accent-500';
</script>

<div class="flex flex-col gap-1">
  <label
    class="font-mono text-[10px] uppercase tracking-[0.12em] text-ink-400"
    for="user-search-{name}">{label}</label
  >
  <div class="relative">
    <input
      id="user-search-{name}"
      type="text"
      class={inputClass}
      value={query}
      autocomplete="off"
      placeholder="Search by name or email…"
      oninput={(e) => onInput((e.target as HTMLInputElement).value)}
    />
    <input type="hidden" {name} value={selectedId} />
    {#if chosen}
      <button
        type="button"
        class="absolute right-2 top-1/2 -translate-y-1/2 rounded px-2 py-0.5 text-xs text-ink-600 hover:bg-ink-100"
        onclick={clear}
      >
        Clear
      </button>
    {/if}
  </div>

  {#if hint}
    <p class="text-[12px] leading-[1.45] text-ink-400">{hint}</p>
  {/if}

  {#if !chosen && results.length > 0}
    <ul
      role="listbox"
      class="max-h-56 overflow-auto rounded-[3px] border border-line bg-white shadow-sm"
    >
      {#each results as r (r.id)}
        <li>
          <button
            type="button"
            role="option"
            aria-selected="false"
            class="block w-full cursor-pointer px-3 py-2 text-left hover:bg-ink-50"
            onclick={() => pick(r)}
          >
            {r.display_name}
          </button>
        </li>
      {/each}
    </ul>
  {:else if !chosen && query.trim().length >= 2 && !loading}
    <p class="text-xs text-ink-600">No matches.</p>
  {/if}
</div>
