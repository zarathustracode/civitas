<script lang="ts">
  import type { Snippet } from 'svelte';

  type Variant = 'primary' | 'secondary' | 'danger' | 'ghost';
  type Size = 'sm' | 'md' | 'lg';

  let {
    variant = 'primary',
    size = 'md',
    type = 'button',
    disabled = false,
    loading = false,
    onclick,
    children,
    ariaLabel
  }: {
    variant?: Variant;
    size?: Size;
    type?: 'button' | 'submit' | 'reset';
    disabled?: boolean;
    loading?: boolean;
    onclick?: (e: MouseEvent) => void;
    children: Snippet;
    ariaLabel?: string;
  } = $props();

  const variantClass: Record<Variant, string> = {
    primary:
      'bg-accent-600 text-white hover:bg-accent-700 disabled:bg-ink-200 disabled:text-ink-400',
    secondary: 'bg-white border border-ink-200 text-ink-800 hover:bg-ink-50 disabled:opacity-50',
    danger: 'bg-oppose-600 text-white hover:opacity-90 disabled:opacity-50',
    ghost: 'bg-transparent text-ink-800 hover:bg-ink-100 disabled:opacity-50'
  };
  const sizeClass: Record<Size, string> = {
    sm: 'px-3 py-1.5 text-sm min-h-[36px]',
    md: 'px-4 py-2 text-base min-h-[44px]',
    lg: 'px-5 py-3 text-lg min-h-[52px]'
  };
</script>

<button
  {type}
  {onclick}
  disabled={disabled || loading}
  aria-label={ariaLabel}
  aria-busy={loading}
  class="inline-flex items-center justify-center gap-2 rounded-md font-medium transition-colors disabled:cursor-not-allowed {variantClass[
    variant
  ]} {sizeClass[size]}"
>
  {#if loading}
    <span aria-hidden="true" class="inline-block animate-pulse">…</span>
  {/if}
  {@render children()}
</button>
