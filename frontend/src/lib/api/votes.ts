import type { Tally, UUID, Vote, VoteChoice } from '$lib/types/domain';
import { apiFetch, CLIENT_BASE_PATH } from './client';

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
 * Browser-only `EventSource` URL for live tally updates: one `tally` event
 * carrying a `TallyUpdate` as soon as it is known, then one per change.
 */
export function tallyStreamUrl(proposalId: UUID): string {
  return `${CLIENT_BASE_PATH}/proposals/${encodeURIComponent(proposalId)}/tally/stream`;
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
