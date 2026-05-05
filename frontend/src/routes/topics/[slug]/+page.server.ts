import { error } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';
import { getTopic } from '$lib/api/topics';
import { listProposals } from '$lib/api/proposals';
import { ApiError } from '$lib/api/errors';

export const load: PageServerLoad = async ({ params, fetch, request }) => {
  try {
    const topic = await getTopic(params.slug, fetch, request.headers);
    const proposals = await listProposals({ topic_id: topic.id }, fetch, request.headers);
    return { topic, proposals };
  } catch (e) {
    if (e instanceof ApiError && e.status === 404) {
      throw error(404, 'Topic not found');
    }
    throw e;
  }
};
