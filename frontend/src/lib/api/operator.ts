import type { OperatorAuditEntry, OperatorOverview, UUID } from '$lib/types/domain';
import { apiFetch } from './client';

/** Deployment-wide counts. Operators only (403 otherwise). */
export async function getOperatorOverview(
  customFetch?: typeof fetch,
  forwardHeaders?: Headers
): Promise<OperatorOverview> {
  return apiFetch<OperatorOverview>('/operator/overview', { fetch: customFetch, forwardHeaders });
}

/**
 * The most recent audit events across every entity, newest first. Pass the
 * last id of a page as `before` to fetch the next, older page.
 */
export async function listOperatorAudit(
  before: UUID | null,
  customFetch?: typeof fetch,
  forwardHeaders?: Headers
): Promise<OperatorAuditEntry[]> {
  return apiFetch<OperatorAuditEntry[]>('/operator/audit', {
    query: { before, limit: OPERATOR_AUDIT_PAGE },
    fetch: customFetch,
    forwardHeaders
  });
}

export const OPERATOR_AUDIT_PAGE = 50;
