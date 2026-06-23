<script lang="ts">
  import type { Snippet } from 'svelte';

  type Tone = 'info' | 'success' | 'warning' | 'error';

  let {
    tone = 'info',
    title,
    children
  }: { tone?: Tone; title?: string; children: Snippet } = $props();

  const toneClass: Record<Tone, string> = {
    info: 'border-accent-600 bg-accent-50 text-ink-900',
    success: 'border-affirm-600 bg-affirm-50 text-ink-900',
    warning: 'border-ochre-600 bg-ochre-50 text-ink-900',
    error: 'border-oppose-600 bg-oppose-50 text-ink-900'
  };
  const role = $derived(tone === 'error' || tone === 'warning' ? 'alert' : 'status');
</script>

<div
  {role}
  aria-live={role === 'alert' ? 'assertive' : 'polite'}
  class="rounded-[3px] border-l-[3px] px-4 py-3 {toneClass[tone]}"
>
  {#if title}
    <p class="font-semibold">{title}</p>
  {/if}
  <div class="text-sm leading-[1.5]">{@render children()}</div>
</div>
