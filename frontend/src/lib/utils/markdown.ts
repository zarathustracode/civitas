/**
 * Minimal, safe Markdown → token tree.
 *
 * No HTML is ever produced: the renderer (`Markdown.svelte`) interpolates every
 * token's text through Svelte, which escapes it, and emits only a fixed set of
 * elements. Link hrefs are restricted to non-executable schemes here, so the
 * renderer never binds a `javascript:` URL. This is deliberately a small subset
 * of CommonMark — headings, paragraphs, bold/italic/inline-code, links,
 * blockquotes, and unordered/ordered lists — covering the shapes a proposal
 * body uses, not a full implementation.
 */

export type Inline =
  | { kind: 'text'; value: string }
  | { kind: 'strong'; value: string }
  | { kind: 'em'; value: string }
  | { kind: 'code'; value: string }
  | { kind: 'link'; value: string; href: string };

export type Block =
  | { kind: 'heading'; level: 2 | 3 | 4; inlines: Inline[] }
  | { kind: 'paragraph'; inlines: Inline[] }
  | { kind: 'list'; ordered: boolean; items: Inline[][] }
  | { kind: 'quote'; inlines: Inline[] };

/** Hrefs that cannot execute script. Anything else stays literal text. */
const SAFE_HREF = /^(https?:\/\/|mailto:|\/)/i;

export function parseInline(src: string): Inline[] {
  const out: Inline[] = [];
  const pushText = (s: string) => {
    if (!s) return;
    const last = out[out.length - 1];
    if (last && last.kind === 'text') last.value += s;
    else out.push({ kind: 'text', value: s });
  };

  let i = 0;
  while (i < src.length) {
    const ch = src.charAt(i);

    // `inline code`
    if (ch === '`') {
      const end = src.indexOf('`', i + 1);
      if (end > i) {
        out.push({ kind: 'code', value: src.slice(i + 1, end) });
        i = end + 1;
        continue;
      }
    }

    // [text](href) — only when the href scheme is safe
    if (ch === '[') {
      const close = src.indexOf(']', i + 1);
      if (close > i && src.charAt(close + 1) === '(') {
        const paren = src.indexOf(')', close + 2);
        if (paren > close) {
          const href = src.slice(close + 2, paren).trim();
          if (SAFE_HREF.test(href)) {
            out.push({ kind: 'link', value: src.slice(i + 1, close), href });
            i = paren + 1;
            continue;
          }
        }
      }
    }

    // **strong** (checked before single-char emphasis)
    if (src.startsWith('**', i)) {
      const end = src.indexOf('**', i + 2);
      if (end > i) {
        out.push({ kind: 'strong', value: src.slice(i + 2, end) });
        i = end + 2;
        continue;
      }
    }

    // *em* or _em_
    if (ch === '*' || ch === '_') {
      const end = src.indexOf(ch, i + 1);
      if (end > i) {
        out.push({ kind: 'em', value: src.slice(i + 1, end) });
        i = end + 1;
        continue;
      }
    }

    pushText(ch);
    i += 1;
  }
  return out;
}

function clampLevel(n: number): 2 | 3 | 4 {
  if (n <= 2) return 2;
  if (n === 3) return 3;
  return 4;
}

export function parseMarkdown(src: string): Block[] {
  const lines = (src ?? '').replace(/\r\n?/g, '\n').split('\n');
  const blocks: Block[] = [];
  let i = 0;

  const isBlockStart = (l: string) =>
    l.trim() === '' ||
    /^#{1,6}\s+/.test(l) ||
    /^>\s?/.test(l) ||
    /^[-*]\s+/.test(l) ||
    /^\d+\.\s+/.test(l);

  while (i < lines.length) {
    const line = lines[i] ?? '';

    if (line.trim() === '') {
      i += 1;
      continue;
    }

    const heading = /^(#{1,6})\s+(.*)$/.exec(line);
    if (heading) {
      const hashes = heading[1] ?? '';
      blocks.push({
        kind: 'heading',
        level: clampLevel(hashes.length),
        inlines: parseInline((heading[2] ?? '').trim())
      });
      i += 1;
      continue;
    }

    if (/^>\s?/.test(line)) {
      const buf: string[] = [];
      while (i < lines.length && /^>\s?/.test(lines[i] ?? '')) {
        buf.push((lines[i] ?? '').replace(/^>\s?/, ''));
        i += 1;
      }
      blocks.push({ kind: 'quote', inlines: parseInline(buf.join(' ').trim()) });
      continue;
    }

    const ordered = /^\d+\.\s+/.test(line);
    if (ordered || /^[-*]\s+/.test(line)) {
      const items: Inline[][] = [];
      while (i < lines.length) {
        const l = lines[i] ?? '';
        const m = ordered ? /^\d+\.\s+(.*)$/.exec(l) : /^[-*]\s+(.*)$/.exec(l);
        if (!m) break;
        items.push(parseInline((m[1] ?? '').trim()));
        i += 1;
      }
      blocks.push({ kind: 'list', ordered, items });
      continue;
    }

    const buf: string[] = [];
    while (i < lines.length) {
      const l = lines[i] ?? '';
      if (isBlockStart(l)) break;
      buf.push(l);
      i += 1;
    }
    blocks.push({ kind: 'paragraph', inlines: parseInline(buf.join(' ').trim()) });
  }

  return blocks;
}
