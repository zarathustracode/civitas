import { describe, it, expect } from 'vitest';
import { parseInline, parseMarkdown } from './markdown';

describe('parseInline', () => {
  it('parses bold, italic, and inline code', () => {
    expect(parseInline('a **b** c')).toEqual([
      { kind: 'text', value: 'a ' },
      { kind: 'strong', value: 'b' },
      { kind: 'text', value: ' c' }
    ]);
    expect(parseInline('_x_')).toEqual([{ kind: 'em', value: 'x' }]);
    expect(parseInline('`y`')).toEqual([{ kind: 'code', value: 'y' }]);
  });

  it('keeps safe links and refuses script schemes', () => {
    expect(parseInline('[ok](https://example.com)')).toEqual([
      { kind: 'link', value: 'ok', href: 'https://example.com' }
    ]);
    // A javascript: URL is never turned into a link — it stays literal text,
    // so the renderer can never bind an executable href.
    const danger = parseInline('[x](javascript:alert(1))');
    expect(danger.some((t) => t.kind === 'link')).toBe(false);
  });

  it('does not emit raw HTML tokens (renderer escapes text)', () => {
    const tokens = parseInline('<img src=x onerror=alert(1)>');
    // Only text/strong/em/code/link tokens exist; angle brackets are plain text.
    expect(tokens.every((t) => ['text', 'strong', 'em', 'code', 'link'].includes(t.kind))).toBe(
      true
    );
  });
});

describe('parseMarkdown', () => {
  it('parses headings, paragraphs, and lists', () => {
    const blocks = parseMarkdown('## Title\n\nA para.\n\n- one\n- two');
    expect(blocks[0]).toMatchObject({ kind: 'heading', level: 2 });
    expect(blocks[1]).toMatchObject({ kind: 'paragraph' });
    const list = blocks[2];
    expect(list).toMatchObject({ kind: 'list', ordered: false });
    if (list && list.kind === 'list') expect(list.items).toHaveLength(2);
  });

  it('clamps heading levels into the 2..4 page hierarchy', () => {
    expect(parseMarkdown('# H1')[0]).toMatchObject({ level: 2 });
    expect(parseMarkdown('###### H6')[0]).toMatchObject({ level: 4 });
  });

  it('parses ordered lists and blockquotes', () => {
    expect(parseMarkdown('1. first\n2. second')[0]).toMatchObject({ kind: 'list', ordered: true });
    expect(parseMarkdown('> quoted')[0]).toMatchObject({ kind: 'quote' });
  });

  it('treats plain text as a single paragraph', () => {
    const blocks = parseMarkdown('just text');
    expect(blocks).toHaveLength(1);
    expect(blocks[0]).toMatchObject({ kind: 'paragraph' });
  });
});
