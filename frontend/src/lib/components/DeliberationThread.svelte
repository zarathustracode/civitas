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
  const stanceClass: Record<Stance, string> = {
    support: 'bg-affirm-600/10 text-affirm-600',
    oppose: 'bg-oppose-600/10 text-oppose-600',
    neutral: 'bg-ink-100 text-ink-800',
    question: 'bg-accent-50 text-accent-700'
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
      class="rounded-md border border-ink-200 bg-white p-3"
      style="margin-left: {Math.min(depth, 4) * 1.25}rem"
    >
      <header class="mb-1 flex flex-wrap items-baseline gap-2 text-sm">
        <span
          class="inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium {stanceClass[
            node.stance
          ]}"
        >
          {stanceLabel[node.stance]}
        </span>
        <span class="font-mono text-xs text-ink-600">{node.author_id.slice(0, 8)}</span>
        <span class="text-xs text-ink-600">{fmtDate(node.created_at)}</span>
        {#if node.edited_at}
          <span class="text-xs italic text-ink-600">(edited)</span>
        {/if}
      </header>
      <p class="whitespace-pre-line text-sm text-ink-900">{visibleBody(node)}</p>
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
