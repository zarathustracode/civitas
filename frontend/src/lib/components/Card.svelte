<script lang="ts">
  import type { Snippet } from 'svelte';
  let { children, padding = 'md' }: { children: Snippet; padding?: 'sm' | 'md' | 'lg' } = $props();
  const padClass = $derived({ sm: 'p-3', md: 'p-4', lg: 'p-6' }[padding]);
</script>

<div class="rounded-lg border border-ink-200 bg-white {padClass}">{@render children()}</div>
