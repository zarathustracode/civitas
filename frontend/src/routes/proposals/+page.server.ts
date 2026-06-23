import type { PageServerLoad } from './$types';
import { listProposalSummaries } from '$lib/api/proposals';
import { listTopics } from '$lib/api/topics';

/**
 * The docket loads every proposal enriched with its tally summary and comment
 * count in a single call, plus topics for name resolution. Filtering happens
 * client-side over the full set so the status chips can show live counts.
 */
export const load: PageServerLoad = async ({ fetch, request }) => {
  const headers = request.headers;
  const [items, topics] = await Promise.all([
    listProposalSummaries({}, fetch, headers),
    listTopics(fetch, headers)
  ]);
  return { items, topics };
};
