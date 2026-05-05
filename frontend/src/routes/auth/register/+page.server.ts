import { fail, redirect } from '@sveltejs/kit';
import type { Actions, PageServerLoad } from './$types';

export const load: PageServerLoad = ({ locals }) => {
  if (locals.currentUser) {
    throw redirect(303, '/proposals');
  }
  return {};
};

export const actions: Actions = {
  default: async ({ request, fetch }) => {
    const form = await request.formData();
    const email = (form.get('email') ?? '').toString().trim();
    const password = (form.get('password') ?? '').toString();
    const display_name = (form.get('display_name') ?? '').toString().trim();

    if (!email || !password || !display_name) {
      return fail(400, { code: 'request.bad', email, display_name });
    }

    const response = await fetch('/api/auth/register', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ email, password, display_name })
    });

    if (!response.ok) {
      let code = 'internal';
      try {
        const body = (await response.json()) as { error?: { code?: string } };
        code = body.error?.code ?? code;
      } catch {
        /* fall through */
      }
      return fail(response.status, { code, email, display_name });
    }

    throw redirect(303, '/auth/verify-email?registered=1');
  }
};
