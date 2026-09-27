import { describe, expect, it } from 'vitest';
import { catalog } from '$lib/server/i18n';
import { createI18n, LOCALES, negotiate, type Messages } from '$lib/i18n';

const english = catalog('en');

const texts = (entry: Messages[string]): string[] =>
  typeof entry === 'string' ? [entry] : Object.values(entry).filter((v) => v !== undefined);

const placeholders = (entry: Messages[string]) =>
  [...new Set(texts(entry).flatMap((t) => [...t.matchAll(/\{(\w+)\}/g)].map((m) => m[1])))].sort();

const markup = (entry: Messages[string]) =>
  texts(entry)
    .flatMap((t) => [...t.matchAll(/<\/?(strong|em|link)>/g)].map((m) => m[0]))
    .sort();

// The plural forms a language reaches with everyday counts: Polish needs
// "few" and "many", Spanish's "many" only starts at a million.
const categoriesInUse = (locale: string) => {
  const rules = new Intl.PluralRules(locale);
  return [...new Set(Array.from({ length: 201 }, (_, n) => rules.select(n)))].sort();
};

describe('message catalogs', () => {
  for (const locale of LOCALES.filter((l) => l !== 'en')) {
    const messages = catalog(locale);

    it(`${locale} has exactly the English keys`, () => {
      expect(Object.keys(messages).sort()).toEqual(Object.keys(english).sort());
    });

    it(`${locale} keeps each message's placeholders and emphasis`, () => {
      for (const [key, entry] of Object.entries(messages)) {
        const source = english[key];
        if (source === undefined) continue;
        expect(placeholders(entry), key).toEqual(placeholders(source));
        expect(markup(entry), key).toEqual(markup(source));
        expect(typeof entry, key).toEqual(typeof source);
      }
    });
  }

  for (const locale of LOCALES) {
    it(`${locale} plurals cover every form a count reaches`, () => {
      const needed = categoriesInUse(locale);
      for (const [key, entry] of Object.entries(catalog(locale))) {
        if (typeof entry === 'string') continue;
        expect(Object.keys(entry).sort(), key).toEqual(expect.arrayContaining(needed));
      }
    });
  }

  it('every key the source names exists', () => {
    const sources = import.meta.glob<string>('/src/**/*.{svelte,ts}', {
      query: '?raw',
      import: 'default',
      eager: true
    });
    const used = new Set<string>();
    for (const [path, text] of Object.entries(sources)) {
      if (path.endsWith('.test.ts')) continue;
      for (const m of text.matchAll(/\b(?:t|rich)\(\s*'([a-z_]+\.[\w.]+)'/g)) used.add(m[1]!);
    }
    expect(used.size).toBeGreaterThan(0);
    expect([...used].filter((key) => !(key in english))).toEqual([]);
  });
});

describe('createI18n', () => {
  const messages: Messages = {
    'x.hello': 'Hello, {name}!',
    'x.votes': {
      one: '{count} głos',
      few: '{count} głosy',
      many: '{count} głosów',
      other: '{count} głosu'
    },
    'x.rich': 'You voted <strong>{choice}</strong>.'
  };
  const pl = createI18n('pl', messages);

  it('fills placeholders and picks the plural form', () => {
    expect(pl.t('x.hello', { name: 'Ala' })).toBe('Hello, Ala!');
    expect([1, 2, 5, 22, 1000].map((count) => pl.t('x.votes', { count }))).toEqual([
      '1 głos',
      '2 głosy',
      '5 głosów',
      '22 głosy',
      '1000 głosów'
    ]);
  });

  it('formats numbers for the locale', () => {
    expect(createI18n('es', messages).t('x.hello', { name: 12345.5 })).toBe('Hello, 12.345,5!');
  });

  it('keeps markup in parameters as text', () => {
    expect(pl.rich('x.rich', { choice: '<strong>Za</strong>' })).toEqual([
      { tag: 'text', text: 'You voted ' },
      { tag: 'strong', text: '<strong>Za</strong>' },
      { tag: 'text', text: '.' }
    ]);
  });

  it('shows an unknown key as itself', () => {
    expect(pl.t('x.missing')).toBe('x.missing');
  });
});

describe('negotiate', () => {
  it('takes the highest-weighted supported language', () => {
    expect(negotiate('de-DE,pl;q=0.8,es;q=0.9')).toBe('es');
    expect(negotiate('pl-PL,pl;q=0.9,en;q=0.5')).toBe('pl');
    expect(negotiate('fr, de;q=0.5')).toBe('en');
    expect(negotiate(null)).toBe('en');
  });
});
