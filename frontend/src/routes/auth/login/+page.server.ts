import { fail, redirect } from '@sveltejs/kit';
import type { Actions, PageServerLoad } from './$types';
import { forwardSetCookie } from '$lib/server/cookieBridge';

export const load: PageServerLoad = ({ locals, url }) => {
  if (locals.currentUser) {
    throw redirect(303, '/proposals');
  }
  return {
    verified: url.searchParams.get('verified') === '1',
    reset: url.searchParams.get('reset') === '1'
  };
};

export const actions: Actions = {
  default: async ({ request, fetch, cookies }) => {
    const form = await request.formData();
    const email = (form.get('email') ?? '').toString().trim();
    const password = (form.get('password') ?? '').toString();

    if (!email || !password) {
      return fail(400, { code: 'request.bad', email });
    }

    const response = await fetch('/api/auth/login', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ email, password })
    });

    if (!response.ok) {
      let code = 'internal';
      try {
        const body = (await response.json()) as { error?: { code?: string } };
        code = body.error?.code ?? code;
      } catch {
        /* not JSON; fall through */
      }
      return fail(response.status, { code, email });
    }

    forwardSetCookie(response, cookies);
    throw redirect(303, '/proposals');
  }
};
