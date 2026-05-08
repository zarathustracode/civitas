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

    // The API echoes a `dev_verification_token` only when the deployment
    // explicitly opted in via DEV_RETURN_VERIFICATION_TOKEN. Forward it as
    // a query param so the verify-email page can pre-fill the input.
    let token = '';
    try {
      const body = (await response.json()) as { dev_verification_token?: string };
      token = body.dev_verification_token ?? '';
    } catch {
      /* the body might be empty in unusual edge cases; ignore */
    }

    const next = token
      ? `/auth/verify-email?registered=1&token=${encodeURIComponent(token)}`
      : '/auth/verify-email?registered=1';
    throw redirect(303, next);
  }
};
