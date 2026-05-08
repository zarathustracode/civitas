import { error, fail } from '@sveltejs/kit';
import type { Actions, PageServerLoad } from './$types';
import { getProposal } from '$lib/api/proposals';
import { getTally, listMyVotes } from '$lib/api/votes';
import { listComments } from '$lib/api/comments';
import { ApiError } from '$lib/api/errors';
import type { VoteChoice } from '$lib/types/domain';

const CHOICES: VoteChoice[] = ['yes', 'no', 'abstain'];

export const load: PageServerLoad = async ({ params, fetch, request, locals }) => {
  try {
    const [proposal, tally, comments, myVotes] = await Promise.all([
      getProposal(params.id, fetch, request.headers),
      getTally(params.id, fetch, request.headers),
      listComments(params.id, fetch, request.headers),
      locals.currentUser ? listMyVotes(params.id, fetch, request.headers) : Promise.resolve(null)
    ]);
    return { proposal, tally, comments, myVotes };
  } catch (e) {
    if (e instanceof ApiError && e.status === 404) {
      throw error(404, 'Proposal not found');
    }
    throw e;
  }
};

export const actions: Actions = {
  vote: async ({ request, fetch, params }) => {
    const form = await request.formData();
    const choice = (form.get('choice') ?? '').toString();
    if (!CHOICES.includes(choice as VoteChoice)) {
      return fail(400, { code: 'request.bad' });
    }

    const response = await fetch(`/api/proposals/${params.id}/votes`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ choice })
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

    return { voted: choice as VoteChoice };
  }
};
