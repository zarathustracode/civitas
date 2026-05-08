import type { NamedUser } from '$lib/types/domain';
import { apiFetch, type QueryParams } from './client';

/**
 * Substring search for active users by display name or email. Auth-only.
 * Returns up to 20 matches; queries shorter than 2 characters return [].
 */
export async function searchUsers(
  q: string,
  customFetch?: typeof fetch
): Promise<NamedUser[]> {
  const query: QueryParams = { q };
  return apiFetch<NamedUser[]>('/users/search', { query, fetch: customFetch });
}
