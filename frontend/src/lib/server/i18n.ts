/**
 * Message catalogs, assembled on the server. Only the request's locale goes
 * to the browser (through the root layout's data), so no catalog weighs on
 * the JavaScript bundle.
 */

import { DEFAULT_LOCALE, LOCALES, type Locale, type Messages } from '$lib/i18n';

const files = import.meta.glob<Messages>('../i18n/messages/*/*.json', {
  eager: true,
  import: 'default'
});

/** One locale's messages as written, keys prefixed with their file's namespace. */
export function catalog(locale: Locale): Messages {
  const out: Messages = {};
  for (const [path, messages] of Object.entries(files)) {
    const match = path.match(/messages\/([a-z]+)\/([\w-]+)\.json$/);
    if (!match || match[1] !== locale) continue;
    for (const [key, value] of Object.entries(messages)) out[`${match[2]}.${key}`] = value;
  }
  return out;
}

const english = catalog(DEFAULT_LOCALE);

// A key not yet translated falls back to English rather than showing its name.
const catalogs = Object.fromEntries(
  LOCALES.map((locale) => [locale, { ...english, ...catalog(locale) }])
) as Record<Locale, Messages>;

export function messagesFor(locale: Locale): Messages {
  return catalogs[locale];
}
