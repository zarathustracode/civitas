import type { PageServerLoad } from './$types';
import { listProposals } from '$lib/api/proposals';
import { listTopics } from '$lib/api/topics';
import { getTally } from '$lib/api/votes';
import type { Tally, Topic } from '$lib/types/domain';

/**
 * The landing page is grounded in real data rather than illustrative figures:
 * the hero "live instrument" shows the soonest-closing open vote with its real
 * tally, and the stat band reports counts we can actually source (open votes,
 * topics in deliberation, topics). Aggregate figures with no backing endpoint
 * (eligible citizens, active delegations) are intentionally omitted.
 */
export const load: PageServerLoad = async ({ fetch, request }) => {
  const headers = request.headers;
  const [voting, deliberation, topics] = await Promise.all([
    listProposals({ status: 'voting' }, fetch, headers),
    listProposals({ status: 'deliberation' }, fetch, headers),
    listTopics(fetch, headers)
  ]);

  // Feature the soonest-closing open vote; fall back to any open vote.
  const featured =
    [...voting]
      .filter((p) => p.voting_ends_at)
      .sort(
        (a, b) =>
          new Date(a.voting_ends_at as string).getTime() -
          new Date(b.voting_ends_at as string).getTime()
      )[0] ??
    voting[0] ??
    null;

  let featuredTally: Tally | null = null;
  if (featured) {
    try {
      featuredTally = await getTally(featured.id, fetch, headers);
    } catch {
      featuredTally = null;
    }
  }

  const featuredTopic: Topic | undefined = featured
    ? topics.find((t) => t.id === featured.topic_id)
    : undefined;

  return {
    featured,
    featuredTally,
    featuredTopicName: featuredTopic?.name ?? null,
    stats: {
      votingOpen: voting.length,
      inDeliberation: deliberation.length,
      topics: topics.length
    }
  };
};
