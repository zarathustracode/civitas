import type { Tally, UUID, Vote, VoteChoice } from '$lib/types/domain';
import { apiFetch } from './client';

export async function castVote(
  proposalId: UUID,
  choice: VoteChoice,
  customFetch?: typeof fetch
): Promise<Vote> {
  return apiFetch<Vote>(`/proposals/${encodeURIComponent(proposalId)}/votes`, {
    body: { choice },
    fetch: customFetch
  });
}

export async function getTally(
  proposalId: UUID,
  customFetch?: typeof fetch,
  forwardHeaders?: Headers
): Promise<Tally> {
  return apiFetch<Tally>(`/proposals/${encodeURIComponent(proposalId)}/tally`, {
    fetch: customFetch,
    forwardHeaders
  });
}

/**
 * The requesting user's full vote-change history on a proposal, newest
 * first. Index 0 (if any) is the active vote. Auth-required: 401 → null.
 */
export async function listMyVotes(
  proposalId: UUID,
  customFetch?: typeof fetch,
  forwardHeaders?: Headers
): Promise<Vote[] | null> {
  return apiFetch<Vote[]>(`/proposals/${encodeURIComponent(proposalId)}/votes/mine`, {
    fetch: customFetch,
    forwardHeaders,
    allowUnauthenticated: true
  });
}
