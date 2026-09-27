/**
 * Translation and locale formatting.
 *
 * Messages live in `messages/<locale>/<namespace>.json` and are addressed as
 * `namespace.key`. The server picks the request's locale (cookie, then
 * Accept-Language), loads that catalog, and hands it to the root layout,
 * which puts an `I18n` in Svelte context. Context rather than a module-level
 * store: the server renders many requests at once, and a shared store would
 * let one visitor's language leak into another's page.
 *
 * Message syntax:
 * - `{name}` placeholders; number values are formatted for the locale.
 * - Plurals are objects keyed by CLDR category, chosen by `count`:
 *   `{ "one": "{count} vote", "other": "{count} votes" }`. Each locale
 *   supplies every category its language uses (Polish: one, few, many,
 *   other).
 * - `<strong>` and `<em>` for emphasis inside a sentence, and `<link>` for
 *   a link's text (the page supplies the address), rendered by `rich()` and
 *   the `Rich` component, never as raw HTML.
 */

import { getContext, setContext } from 'svelte';

export const LOCALES = ['en', 'es', 'pl'] as const;
export type Locale = (typeof LOCALES)[number];
export const DEFAULT_LOCALE: Locale = 'en';
export const LOCALE_COOKIE = 'civitas_locale';

/** Each language named in itself, for the language picker. */
export const LOCALE_NAMES: Record<Locale, string> = {
  en: 'English',
  es: 'Español',
  pl: 'Polski'
};

export type PluralForms = Partial<Record<Intl.LDMLPluralRule, string>> & { other: string };
export type Messages = Record<string, string | PluralForms>;
export type Params = Record<string, string | number>;
export type RichPart = { tag: 'text' | 'strong' | 'em' | 'link'; text: string };

export function isLocale(value: unknown): value is Locale {
  return typeof value === 'string' && (LOCALES as readonly string[]).includes(value);
}

/** The best supported locale for an Accept-Language header. */
export function negotiate(header: string | null | undefined): Locale {
  if (!header) return DEFAULT_LOCALE;
  const ranked = header
    .split(',')
    .map((part) => {
      const [tag, ...attrs] = part.trim().split(';');
      const q = attrs.map((a) => a.trim()).find((a) => a.startsWith('q='));
      return { lang: (tag ?? '').split('-')[0]?.toLowerCase(), q: q ? Number(q.slice(2)) : 1 };
    })
    .filter((r) => r.lang && r.q > 0)
    .sort((a, b) => b.q - a.q);
  return ranked.map((r) => r.lang).find(isLocale) ?? DEFAULT_LOCALE;
}

export interface I18n {
  readonly locale: Locale;
  /** The message for `key`, with `params` filled in. Unknown keys render as themselves. */
  t(key: string, params?: Params): string;
  has(key: string): boolean;
  /** Like `t`, split into plain and emphasized parts for the `Rich` component. */
  rich(key: string, params?: Params): RichPart[];
  number(value: number, options?: Intl.NumberFormatOptions): string;
  date(value: string | Date, options?: Intl.DateTimeFormatOptions): string;
}

const MARKUP = /<(strong|em|link)>(.*?)<\/\1>/g;

export function createI18n(locale: Locale, messages: Messages): I18n {
  const plurals = new Intl.PluralRules(locale);
  const numbers = new Intl.NumberFormat(locale);

  const template = (key: string, params?: Params): string => {
    const entry = messages[key];
    if (entry === undefined) return key;
    if (typeof entry === 'string') return entry;
    const count = Number(params?.count ?? 0);
    return entry[plurals.select(count)] ?? entry.other;
  };

  const fill = (text: string, params?: Params): string =>
    text.replace(/\{(\w+)\}/g, (whole, name: string) => {
      const value = params?.[name];
      if (value === undefined) return whole;
      return typeof value === 'number' ? numbers.format(value) : value;
    });

  return {
    locale,
    t: (key, params) => fill(template(key, params), params),
    has: (key) => key in messages,
    rich(key, params) {
      // Split before filling, so a parameter (a display name, say) can never
      // introduce markup of its own.
      const source = template(key, params);
      const parts: RichPart[] = [];
      let last = 0;
      for (const m of source.matchAll(MARKUP)) {
        if (m.index > last) parts.push({ tag: 'text', text: source.slice(last, m.index) });
        parts.push({ tag: m[1] as RichPart['tag'], text: m[2] ?? '' });
        last = m.index + m[0].length;
      }
      if (last < source.length) parts.push({ tag: 'text', text: source.slice(last) });
      return parts.map((p) => ({ ...p, text: fill(p.text, params) }));
    },
    number: (value, options) =>
      options ? new Intl.NumberFormat(locale, options).format(value) : numbers.format(value),
    date: (value, options) =>
      new Intl.DateTimeFormat(locale, options ?? { dateStyle: 'medium' }).format(new Date(value))
  };
}

const KEY = Symbol('i18n');

/**
 * Provide translations to the component tree. `current` is read on every
 * call, so a component re-renders when the locale or catalog changes.
 */
export function setI18n(current: () => I18n): void {
  setContext<I18n>(KEY, {
    get locale() {
      return current().locale;
    },
    t: (key, params) => current().t(key, params),
    has: (key) => current().has(key),
    rich: (key, params) => current().rich(key, params),
    number: (value, options) => current().number(value, options),
    date: (value, options) => current().date(value, options)
  });
}

export function getI18n(): I18n {
  const i18n = getContext<I18n | undefined>(KEY);
  if (!i18n) throw new Error('getI18n() called outside the root layout');
  return i18n;
}
