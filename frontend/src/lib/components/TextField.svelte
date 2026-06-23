<script lang="ts">
  import type { HTMLInputAttributes } from 'svelte/elements';

  let {
    name,
    label,
    type = 'text',
    value = $bindable(''),
    placeholder,
    autocomplete,
    required = false,
    disabled = false,
    error,
    hint,
    minlength,
    multiline = false,
    rows = 4
  }: {
    name: string;
    label: string;
    type?: 'text' | 'email' | 'password' | 'url';
    value?: string;
    placeholder?: string;
    autocomplete?: HTMLInputAttributes['autocomplete'];
    required?: boolean;
    disabled?: boolean;
    error?: string;
    hint?: string;
    minlength?: number;
    multiline?: boolean;
    rows?: number;
  } = $props();

  const id = $derived(`f_${name}`);
  const describedBy = $derived(error ? `${id}_err` : hint ? `${id}_hint` : undefined);
  const inputClass =
    'w-full rounded-[3px] border border-line bg-white px-3.5 py-[11px] text-[15px] text-ink-900 placeholder:text-ink-400 focus:border-accent-500 focus:outline-2 focus:outline-offset-2 focus:outline-accent-500 disabled:bg-ink-50 disabled:text-ink-400';
</script>

<div class="flex flex-col gap-1.5">
  <label for={id} class="font-mono text-[10px] uppercase tracking-[0.12em] text-ink-400">
    {label}
    {#if required}<span class="text-oppose-600" aria-hidden="true">*</span>{/if}
  </label>

  {#if multiline}
    <textarea
      {id}
      {name}
      bind:value
      {placeholder}
      {required}
      {disabled}
      {minlength}
      {rows}
      aria-describedby={describedBy}
      aria-invalid={error ? 'true' : undefined}
      class={inputClass}
    ></textarea>
  {:else}
    <input
      {id}
      {name}
      {type}
      bind:value
      {placeholder}
      {autocomplete}
      {required}
      {disabled}
      {minlength}
      aria-describedby={describedBy}
      aria-invalid={error ? 'true' : undefined}
      class={inputClass}
    />
  {/if}

  {#if hint && !error}
    <p id="{id}_hint" class="text-[12px] leading-[1.45] text-ink-400">{hint}</p>
  {/if}
  {#if error}
    <p id="{id}_err" class="text-[12px] text-oppose-600">{error}</p>
  {/if}
</div>
