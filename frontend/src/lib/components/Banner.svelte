<script lang="ts">
  import type { Snippet } from 'svelte';

  type Tone = 'info' | 'success' | 'warning' | 'error';

  let {
    tone = 'info',
    title,
    children
  }: { tone?: Tone; title?: string; children: Snippet } = $props();

  const toneClass: Record<Tone, string> = {
    info: 'bg-accent-50 border-accent-500 text-ink-900',
    success: 'bg-green-50 border-affirm-600 text-ink-900',
    warning: 'bg-yellow-50 border-yellow-500 text-ink-900',
    error: 'bg-red-50 border-oppose-600 text-ink-900'
  };
  const role = $derived(tone === 'error' || tone === 'warning' ? 'alert' : 'status');
</script>

<div
  {role}
  aria-live={role === 'alert' ? 'assertive' : 'polite'}
  class="rounded-md border-l-4 px-4 py-3 {toneClass[tone]}"
>
  {#if title}
    <p class="font-semibold">{title}</p>
  {/if}
  <div class="text-sm">{@render children()}</div>
</div>
