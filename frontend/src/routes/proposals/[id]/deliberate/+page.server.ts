import { fail, redirect } from '@sveltejs/kit';
import type { Actions, PageServerLoad } from './$types';
import { getProposal } from '$lib/api/proposals';
import { listComments } from '$lib/api/comments';
import type { Stance } from '$lib/types/domain';

const VALID_STANCES: Stance[] = ['support', 'oppose', 'neutral', 'question'];

export const load: PageServerLoad = async ({ params, fetch, request }) => {
  const [proposal, comments] = await Promise.all([
    getProposal(params.id, fetch, request.headers),
    listComments(params.id, fetch, request.headers)
  ]);
  return { proposal, comments };
};

export const actions: Actions = {
  post: async ({ request, fetch, params, locals }) => {
    if (!locals.currentUser) {
      throw redirect(303, '/auth/login');
    }
    const form = await request.formData();
    const body = (form.get('body') ?? '').toString().trim();
    const stance = (form.get('stance') ?? '').toString();
    const parent_id = (form.get('parent_id') ?? '').toString() || undefined;

    if (!body || !VALID_STANCES.includes(stance as Stance)) {
      return fail(400, { code: 'request.bad', body, stance });
    }

    const response = await fetch(`/api/proposals/${params.id}/comments`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ body, stance, parent_id })
    });
    if (!response.ok) {
      let code = 'internal';
      try {
        const errBody = (await response.json()) as { error?: { code?: string } };
        code = errBody.error?.code ?? code;
      } catch {
        /* fall through */
      }
      return fail(response.status, { code });
    }
    return { posted: true };
  }
};
