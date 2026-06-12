import { fail, redirect } from '@sveltejs/kit';
import type { Actions, PageServerLoad } from './$types';

export const load: PageServerLoad = ({ url }) => {
  return {
    prefilledToken: url.searchParams.get('token') ?? ''
  };
};

export const actions: Actions = {
  default: async ({ request, fetch }) => {
    const form = await request.formData();
    const token = (form.get('token') ?? '').toString().trim();
    const newPassword = (form.get('new_password') ?? '').toString();
    if (!token || !newPassword) return fail(400, { code: 'request.bad' });

    const response = await fetch('/api/auth/password-reset/complete', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ token, new_password: newPassword })
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

    throw redirect(303, '/auth/login?reset=1');
  }
};
