import type { Topic } from '$lib/types/domain';
import { apiFetch } from './client';

export async function listTopics(
  customFetch?: typeof fetch,
  forwardHeaders?: Headers
): Promise<Topic[]> {
  return apiFetch<Topic[]>('/topics', { fetch: customFetch, forwardHeaders });
}

export async function getTopic(
  slug: string,
  customFetch?: typeof fetch,
  forwardHeaders?: Headers
): Promise<Topic> {
  return apiFetch<Topic>(`/topics/${encodeURIComponent(slug)}`, {
    fetch: customFetch,
    forwardHeaders
  });
}

export interface CreateTopicInput {
  slug: string;
  name: string;
  description?: string;
}

export async function createTopic(
  input: CreateTopicInput,
  customFetch?: typeof fetch
): Promise<Topic> {
  return apiFetch<Topic>('/topics', { body: input, fetch: customFetch });
}
