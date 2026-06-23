import type {
  AuditEntry,
  Proposal,
  ProposalListItem,
  ProposalStatus,
  UUID
} from '$lib/types/domain';
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

/**
 * Enriched docket listing: proposals plus their live tally summary and
 * visible comment count, in one request (`GET /proposals/summaries`). With no
 * filter it returns every status, so the list view can group/filter client
 * side. Avoids fanning out a tally + comments request per row.
 */
export async function listProposalSummaries(
  params: ListProposalsParams = {},
  customFetch?: typeof fetch,
  forwardHeaders?: Headers
): Promise<ProposalListItem[]> {
  const query: QueryParams = { topic_id: params.topic_id, status: params.status };
  return apiFetch<ProposalListItem[]>('/proposals/summaries', {
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

/**
 * Audit timeline for a proposal: lifecycle events (creation, status
 * transitions) newest first. Public read; the data is not sensitive
 * because the project's transparency commitment makes it observable.
 */
export async function listProposalAudit(
  id: UUID,
  customFetch?: typeof fetch,
  forwardHeaders?: Headers
): Promise<AuditEntry[]> {
  return apiFetch<AuditEntry[]>(`/proposals/${encodeURIComponent(id)}/audit`, {
    fetch: customFetch,
    forwardHeaders
  });
}
