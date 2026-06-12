import { fail, redirect } from '@sveltejs/kit';
import type { Actions, PageServerLoad } from './$types';

export const load: PageServerLoad = ({ url }) => {
  return {
    registered: url.searchParams.get('registered') === '1',
    prefilledToken: url.searchParams.get('token') ?? ''
  };
};

export const actions: Actions = {
  verify: async ({ request, fetch }) => {
    const form = await request.formData();
    const token = (form.get('token') ?? '').toString().trim();
    if (!token) return fail(400, { code: 'request.bad' });

    const response = await fetch('/api/auth/verify-email', {
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
        /* fall through */
      }
      return fail(response.status, { code });
    }

    throw redirect(303, '/auth/login?verified=1');
  },

  resend: async ({ request, fetch }) => {
    const form = await request.formData();
    const email = (form.get('email') ?? '').toString().trim();
    if (!email) return fail(400, { code: 'request.bad' });

    const response = await fetch('/api/auth/resend-verification', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ email })
    });

    if (!response.ok) {
      let code = 'internal';
      try {
        const body = (await response.json()) as { error?: { code?: string } };
        code = body.error?.code ?? code;
      } catch {
        /* fall through */
      }
      return fail(response.status, { code });
    }

    // 202 regardless of whether the address exists — mirror that.
    return { resent: true };
  }
};
