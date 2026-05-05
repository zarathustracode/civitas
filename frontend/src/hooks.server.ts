/**
 * Server-side hooks.
 *
 * On every request, populate `event.locals.currentUser` by asking the API
 * who we are. Connection failures are downgraded to anonymous so a flaky
 * backend does not return a 500 on every page load.
 */

import type { Handle } from '@sveltejs/kit';
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
