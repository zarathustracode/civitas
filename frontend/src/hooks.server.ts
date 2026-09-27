/**
 * Server-side hooks.
 *
 * On every request, pick the locale and populate `event.locals.currentUser`
 * by asking the API who we are. Connection failures are downgraded to
 * anonymous so a flaky backend does not return a 500 on every page load.
 */

import type { Handle, HandleFetch } from '@sveltejs/kit';
import { getCurrentUser } from '$lib/api/auth';
import { createI18n, isLocale, negotiate, LOCALE_COOKIE } from '$lib/i18n';
import { messagesFor } from '$lib/server/i18n';

export const handle: Handle = async ({ event, resolve }) => {
  // An explicit choice (the footer's language picker) wins over the browser's.
  const chosen = event.cookies.get(LOCALE_COOKIE);
  const locale = isLocale(chosen)
    ? chosen
    : negotiate(event.request.headers.get('accept-language'));
  event.locals.locale = locale;
  event.locals.i18n = createI18n(locale, messagesFor(locale));

  try {
    event.locals.currentUser = await getCurrentUser(event.fetch, event.request.headers);
  } catch (e) {
    console.warn('hooks.server: /auth/me failed, treating as anonymous', e);
    event.locals.currentUser = null;
  }
  return resolve(event, {
    transformPageChunk: ({ html }) => html.replace('%lang%', locale)
  });
};

/**
 * Server-side API calls otherwise reach the backend from this server's IP
 * and user agent, which would put every user in one rate-limit bucket and
 * record the Node runtime on every session. Forward the real client address
 * and browser user agent; the API only trusts the address when TRUST_PROXY
 * is set there.
 */
export const handleFetch: HandleFetch = async ({ event, request, fetch }) => {
  try {
    request.headers.set('x-forwarded-for', event.getClientAddress());
  } catch {
    // No client address available (e.g. prerendering) — send as-is.
  }
  const userAgent = event.request.headers.get('user-agent');
  if (userAgent) request.headers.set('user-agent', userAgent);
  return fetch(request);
};
