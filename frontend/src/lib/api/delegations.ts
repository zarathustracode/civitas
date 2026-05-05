import type { Delegation, UUID } from '$lib/types/domain';
import { apiFetch } from './client';

export async function listMyDelegations(
  customFetch?: typeof fetch,
  forwardHeaders?: Headers
): Promise<Delegation[]> {
  return apiFetch<Delegation[]>('/delegations', { fetch: customFetch, forwardHeaders });
}

export interface CreateDelegationInput {
  topic_id: UUID;
  delegate_id: UUID;
}

export async function createDelegation(
  input: CreateDelegationInput,
  customFetch?: typeof fetch
): Promise<Delegation> {
  return apiFetch<Delegation>('/delegations', { body: input, fetch: customFetch });
}

export async function revokeDelegation(id: UUID, customFetch?: typeof fetch): Promise<void> {
  await apiFetch(`/delegations/${encodeURIComponent(id)}`, {
    method: 'DELETE',
    fetch: customFetch
  });
}
