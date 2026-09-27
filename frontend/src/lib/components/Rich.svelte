<script lang="ts">
  import type { RichPart } from '$lib/i18n';

  // Renders `i18n.rich(...)` output: emphasis or a link inside a translated
  // sentence, without raw HTML. Style <strong>/<em> from the parent (e.g.
  // [&_strong]:...); a <link> part becomes an anchor to `href`.
  let {
    parts,
    href,
    rel,
    linkClass
  }: { parts: RichPart[]; href?: string; rel?: string; linkClass?: string } = $props();
</script>

{#each parts as part, i (i)}{#if part.tag === 'strong'}<strong>{part.text}</strong
    >{:else if part.tag === 'em'}<em>{part.text}</em>{:else if part.tag === 'link'}<a
      {href}
      {rel}
      class={linkClass}>{part.text}</a
    >{:else}{part.text}{/if}{/each}
