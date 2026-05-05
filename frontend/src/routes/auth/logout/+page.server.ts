import { redirect } from '@sveltejs/kit';
import type { Actions, PageServerLoad } from './$types';
import { forwardSetCookie } from '$lib/server/cookieBridge';

export const load: PageServerLoad = () => {
  // POST-only.
  throw redirect(303, '/');
};

export const actions: Actions = {
  default: async ({ fetch, cookies }) => {
    const response = await fetch('/api/auth/logout', { method: 'POST' });
    if (response.ok) {
      forwardSetCookie(response, cookies);
    }
    throw redirect(303, '/');
  }
};
