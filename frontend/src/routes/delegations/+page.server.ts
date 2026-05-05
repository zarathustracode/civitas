import { fail, redirect } from '@sveltejs/kit';
import type { Actions, PageServerLoad } from './$types';
import { listMyDelegations } from '$lib/api/delegations';
import { listTopics } from '$lib/api/topics';

export const load: PageServerLoad = async ({ fetch, request, locals }) => {
  if (!locals.currentUser) {
    throw redirect(303, '/auth/login');
  }
  const [mine, topics] = await Promise.all([
    listMyDelegations(fetch, request.headers),
    listTopics(fetch, request.headers)
  ]);
  return { mine, topics };
};

export const actions: Actions = {
  create: async ({ request, fetch }) => {
    const form = await request.formData();
    const topic_id = (form.get('topic_id') ?? '').toString();
    const delegate_id = (form.get('delegate_id') ?? '').toString();
    if (!topic_id || !delegate_id) return fail(400, { code: 'request.bad' });

    const response = await fetch('/api/delegations', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ topic_id, delegate_id })
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
    return { created: true };
  },

  revoke: async ({ request, fetch }) => {
    const form = await request.formData();
    const id = (form.get('id') ?? '').toString();
    if (!id) return fail(400, { code: 'request.bad' });

    const response = await fetch(`/api/delegations/${encodeURIComponent(id)}`, {
      method: 'DELETE'
    });
    if (!response.ok) {
      return fail(response.status, { code: 'internal' });
    }
    return { revoked: true };
  }
};
