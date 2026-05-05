import type { Comment, Stance, UUID } from '$lib/types/domain';
import { apiFetch } from './client';

export async function listComments(
  proposalId: UUID,
  customFetch?: typeof fetch,
  forwardHeaders?: Headers
): Promise<Comment[]> {
  return apiFetch<Comment[]>(`/proposals/${encodeURIComponent(proposalId)}/comments`, {
    fetch: customFetch,
    forwardHeaders
  });
}

export interface CreateCommentInput {
  parent_id?: UUID;
  body: string;
  stance: Stance;
}

export async function createComment(
  proposalId: UUID,
  input: CreateCommentInput,
  customFetch?: typeof fetch
): Promise<Comment> {
  return apiFetch<Comment>(`/proposals/${encodeURIComponent(proposalId)}/comments`, {
    body: input,
    fetch: customFetch
  });
}

export async function deleteComment(id: UUID, customFetch?: typeof fetch): Promise<void> {
  await apiFetch(`/comments/${encodeURIComponent(id)}`, { method: 'DELETE', fetch: customFetch });
}
