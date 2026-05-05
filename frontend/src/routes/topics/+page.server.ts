import type { PageServerLoad } from './$types';
import { listTopics } from '$lib/api/topics';

export const load: PageServerLoad = async ({ fetch, request }) => {
  const topics = await listTopics(fetch, request.headers);
  return { topics };
};
