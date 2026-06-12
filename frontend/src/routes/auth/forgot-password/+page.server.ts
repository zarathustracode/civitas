import { fail } from '@sveltejs/kit';
import type { Actions } from './$types';

export const actions: Actions = {
  default: async ({ request, fetch }) => {
    const form = await request.formData();
    const email = (form.get('email') ?? '').toString().trim();
    if (!email) return fail(400, { code: 'request.bad', email });

    const response = await fetch('/api/auth/password-reset/request', {
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
        /* not JSON; fall through */
      }
      return fail(response.status, { code, email });
    }

    // The API answers 202 whether or not the address exists — mirror that
    // here and show the same confirmation either way.
    return { sent: true, email };
  }
};
