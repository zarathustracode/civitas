import type { I18n } from '$lib/i18n';

/**
 * Stable API error codes, mirrored from `civitas-api::error::ApiError`.
 *
 * The frontend switches on `code`; `message` is for fallback display. Adding
 * a new code on the backend requires adding it here too.
 */

export type ApiErrorCode =
  // Authentication / authorization
  | 'auth.unauthorized'
  | 'auth.invalid_credentials'
  | 'auth.not_verified'
  | 'auth.token_invalid'
  | 'auth.forbidden'
  // Validation
  | 'request.bad'
  | 'request.invalid_email'
  | 'request.password_too_short'
  | 'request.display_name_required'
  | 'request.reason_required'
  // Conflicts
  | 'user.email_taken'
  | 'user.phone_taken'
  | 'topic.slug_taken'
  | 'delegation.cycle'
  | 'delegation.self'
  | 'delegation.already_active'
  // Domain
  | 'not_found'
  | 'proposal.invalid_transition'
  | 'proposal.voting_window_required'
  | 'proposal.voting_window_invalid'
  | 'vote.outside_window'
  | 'vote.proposal_not_in_voting'
  | 'comment.not_allowed_in_status'
  | 'comment.parent_mismatch'
  // Other
  | 'rate_limited'
  | 'internal';

export class ApiError extends Error {
  readonly code: ApiErrorCode | string;
  readonly status: number;

  constructor(code: ApiErrorCode | string, message: string, status: number) {
    super(message);
    this.name = 'ApiError';
    this.code = code;
    this.status = status;
  }
}

/**
 * Plain-language message for an error code, from the `errors` namespace.
 * Codes without a message of their own get the generic one; the raw code
 * is never shown.
 */
export function friendlyMessage(err: ApiError, i18n: I18n): string {
  const key = `errors.${err.code}`;
  return i18n.t(i18n.has(key) ? key : 'errors.generic');
}
