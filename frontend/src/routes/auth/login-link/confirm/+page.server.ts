import { fail, redirect } from '@sveltejs/kit';
import type { Actions, PageServerLoad } from './$types';
import { forwardSetCookie } from '$lib/server/cookieBridge';

// Arriving here spends nothing: mail scanners prefetch links, so the token
// is only redeemed when the reader presses the button (a POST).
export const load: PageServerLoad = ({ url }) => {
  return { token: url.searchParams.get('token') ?? '' };
};

export const actions: Actions = {
  default: async ({ request, fetch, cookies }) => {
    const form = await request.formData();
    const token = (form.get('token') ?? '').toString().trim();
    if (!token) return fail(400, { code: 'auth.token_invalid' });

    const response = await fetch('/api/auth/login-link/complete', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ token })
    });

    if (!response.ok) {
      let code = 'internal';
      try {
        const body = (await response.json()) as { error?: { code?: string } };
        code = body.error?.code ?? code;
      } catch {
        /* not JSON; fall through */
      }
      return fail(response.status, { code });
    }

    forwardSetCookie(response, cookies);
    throw redirect(303, '/proposals');
  }
};
