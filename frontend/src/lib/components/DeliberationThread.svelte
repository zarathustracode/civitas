<script lang="ts">
  import type { Comment, Stance } from '$lib/types/domain';

  let { comments }: { comments: Comment[] } = $props();

  type Node = Comment & { children: Node[] };

  const tree = $derived.by<Node[]>(() => {
    const byId = new Map<string, Node>();
    const roots: Node[] = [];
    for (const c of comments) {
      byId.set(c.id, { ...c, children: [] });
    }
    for (const node of byId.values()) {
      if (node.parent_id) {
        const parent = byId.get(node.parent_id);
        if (parent) parent.children.push(node);
        else roots.push(node);
      } else {
        roots.push(node);
      }
    }
    return roots;
  });

  const stanceLabel: Record<Stance, string> = {
    support: 'Support',
    oppose: 'Oppose',
    neutral: 'Neutral',
    question: 'Question'
  };
  const stanceMeta: Record<Stance, { text: string; bg: string }> = {
    support: { text: 'text-affirm-600', bg: 'bg-affirm-600' },
    oppose: { text: 'text-oppose-600', bg: 'bg-oppose-600' },
    neutral: { text: 'text-ink-600', bg: 'bg-ink-400' },
    question: { text: 'text-accent-600', bg: 'bg-accent-600' }
  };

  function fmtDate(s: string): string {
    return new Date(s).toLocaleString();
  }

  function visibleBody(c: Comment): string {
    if (c.deleted_at) return '[deleted by author]';
    if (c.hidden_at) return `[hidden by moderator: ${c.hidden_reason ?? 'no reason given'}]`;
    return c.body;
  }
</script>

<ul class="space-y-3">
  {#each tree as node (node.id)}
    {@render commentNode(node, 0)}
  {/each}
</ul>

{#snippet commentNode(node: Node, depth: number)}
  <li>
    <article
      class="rounded border border-line bg-card p-4"
      style="margin-left: {Math.min(depth, 4) * 1.25}rem"
    >
      <header class="mb-2 flex flex-wrap items-center gap-2.5">
        <span
          class="flex h-[26px] w-[26px] flex-none items-center justify-center rounded-full font-serif text-[11px] font-semibold text-white {stanceMeta[
            node.stance
          ].bg}"
        >
          {node.author_id.slice(0, 2).toUpperCase()}
        </span>
        <span class="font-mono text-[12px] text-ink-600">{node.author_id.slice(0, 8)}</span>
        <span
          class="inline-flex rounded-full px-2 py-0.5 font-mono text-[9px] uppercase tracking-[0.1em] {stanceMeta[
            node.stance
          ].text}"
          style="background:rgba(0,0,0,0.04);">{stanceLabel[node.stance]}</span
        >
        <span class="ml-auto font-mono text-[11px] text-ink-400">{fmtDate(node.created_at)}</span>
        {#if node.edited_at}
          <span class="font-mono text-[11px] italic text-ink-400">(edited)</span>
        {/if}
      </header>
      <p class="whitespace-pre-line font-serif text-[16px] leading-[1.55] text-ink-900">
        {visibleBody(node)}
      </p>
    </article>
    {#if node.children.length > 0}
      <ul class="mt-3 space-y-3">
        {#each node.children as child (child.id)}
          {@render commentNode(child, depth + 1)}
        {/each}
      </ul>
    {/if}
  </li>
{/snippet}
