import type { Proposal, ProposalStatus, UUID } from '$lib/types/domain';
import { apiFetch, type QueryParams } from './client';

export interface ListProposalsParams {
  topic_id?: UUID;
  status?: ProposalStatus;
}

export async function listProposals(
  params: ListProposalsParams = {},
  customFetch?: typeof fetch,
  forwardHeaders?: Headers
): Promise<Proposal[]> {
  const query: QueryParams = { topic_id: params.topic_id, status: params.status };
  return apiFetch<Proposal[]>('/proposals', {
    query,
    fetch: customFetch,
    forwardHeaders
  });
}

export async function getProposal(
  id: UUID,
  customFetch?: typeof fetch,
  forwardHeaders?: Headers
): Promise<Proposal> {
  return apiFetch<Proposal>(`/proposals/${encodeURIComponent(id)}`, {
    fetch: customFetch,
    forwardHeaders
  });
}

export interface CreateProposalInput {
  topic_id: UUID;
  title: string;
  summary: string;
  body: string;
}

export async function createProposal(
  input: CreateProposalInput,
  customFetch?: typeof fetch
): Promise<Proposal> {
  return apiFetch<Proposal>('/proposals', { body: input, fetch: customFetch });
}

export interface TransitionStatusInput {
  target: ProposalStatus;
  voting_starts_at?: string;
  voting_ends_at?: string;
}

export async function transitionStatus(
  id: UUID,
  input: TransitionStatusInput,
  customFetch?: typeof fetch
): Promise<Proposal> {
  return apiFetch<Proposal>(`/proposals/${encodeURIComponent(id)}/status`, {
    body: input,
    fetch: customFetch
  });
}
