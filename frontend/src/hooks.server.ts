/**
 * Server-side hooks.
 *
 * On every request, populate `event.locals.currentUser` by asking the API
 * who we are. Connection failures are downgraded to anonymous so a flaky
 * backend does not return a 500 on every page load.
 */

import type { Handle, HandleFetch } from '@sveltejs/kit';
import { getCurrentUser } from '$lib/api/auth';

export const handle: Handle = async ({ event, resolve }) => {
  try {
    event.locals.currentUser = await getCurrentUser(event.fetch, event.request.headers);
  } catch (e) {
    console.warn('hooks.server: /auth/me failed, treating as anonymous', e);
    event.locals.currentUser = null;
  }
  return resolve(event);
};

/**
 * Server-side API calls otherwise reach the backend from this server's IP,
 * which would put every user in one rate-limit bucket. Forward the real
 * client address; the API only trusts it when TRUST_PROXY is set there.
 */
export const handleFetch: HandleFetch = async ({ event, request, fetch }) => {
  try {
    request.headers.set('x-forwarded-for', event.getClientAddress());
  } catch {
    // No client address available (e.g. prerendering) — send as-is.
  }
  return fetch(request);
};
