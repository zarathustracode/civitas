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
