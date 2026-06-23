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
    secondary: 'border border-line bg-white text-ink-900 hover:border-ink-400 disabled:opacity-50',
    danger: 'bg-oppose-600 text-white hover:opacity-90 disabled:opacity-50',
    ghost: 'bg-transparent text-ink-600 hover:bg-ink-100 disabled:opacity-50'
  };
  const sizeClass: Record<Size, string> = {
    sm: 'min-h-[36px] px-3.5 py-2 text-[13px]',
    md: 'min-h-[44px] px-5 py-2.5 text-[15px]',
    lg: 'min-h-[52px] px-6 py-3 text-[17px]'
  };
</script>

<button
  {type}
  {onclick}
  disabled={disabled || loading}
  aria-label={ariaLabel}
  aria-busy={loading}
  class="inline-flex items-center justify-center gap-2 rounded-[3px] font-semibold transition-colors disabled:cursor-not-allowed {variantClass[
    variant
  ]} {sizeClass[size]}"
>
  {#if loading}
    <span aria-hidden="true" class="inline-block animate-pulse">…</span>
  {/if}
  {@render children()}
</button>
