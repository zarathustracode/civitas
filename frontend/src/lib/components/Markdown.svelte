<script lang="ts">
  import { parseMarkdown, type Inline } from '$lib/utils/markdown';

  let { source }: { source: string } = $props();

  const blocks = $derived(parseMarkdown(source));
</script>

{#snippet inline(
  items: Inline[]
)}{#each items as t, i (i)}{#if t.kind === 'text'}{t.value}{:else if t.kind === 'strong'}<strong
        class="font-semibold">{t.value}</strong
      >{:else if t.kind === 'em'}<em>{t.value}</em>{:else if t.kind === 'code'}<code
        class="rounded bg-ink-100 px-1 py-0.5 font-mono text-[0.9em]">{t.value}</code
      >{:else if t.kind === 'link'}<a
        href={t.href}
        class="text-accent-600 underline underline-offset-2 hover:text-accent-700"
        rel="noopener noreferrer">{t.value}</a
      >{/if}{/each}{/snippet}

<div class="font-serif text-[19px] leading-[1.62] text-ink-900">
  {#each blocks as b, i (i)}
    {#if b.kind === 'heading'}
      {#if b.level === 2}
        <h2 class="mt-8 font-serif text-[26px] font-semibold tracking-[-0.01em] first:mt-0">
          {@render inline(b.inlines)}
        </h2>
      {:else if b.level === 3}
        <h3 class="mt-6 font-serif text-[21px] font-semibold first:mt-0">
          {@render inline(b.inlines)}
        </h3>
      {:else}
        <h4 class="mt-5 font-serif text-[18px] font-semibold first:mt-0">
          {@render inline(b.inlines)}
        </h4>
      {/if}
    {:else if b.kind === 'paragraph'}
      <p class="mt-4 first:mt-0">{@render inline(b.inlines)}</p>
    {:else if b.kind === 'quote'}
      <blockquote class="mt-5 border-l-2 border-ink-900 pl-5 italic text-ink-800 first:mt-0">
        {@render inline(b.inlines)}
      </blockquote>
    {:else if b.kind === 'list'}
      {#if b.ordered}
        <ol class="mt-4 list-decimal space-y-1.5 pl-6 first:mt-0">
          {#each b.items as it, j (j)}<li>{@render inline(it)}</li>{/each}
        </ol>
      {:else}
        <ul class="mt-4 list-disc space-y-1.5 pl-6 first:mt-0">
          {#each b.items as it, j (j)}<li>{@render inline(it)}</li>{/each}
        </ul>
      {/if}
    {/if}
  {/each}
</div>
