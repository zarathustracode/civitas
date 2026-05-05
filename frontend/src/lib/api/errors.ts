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
 * Plain-language messages for the most common error codes. Fall back to the
 * server's message for anything we haven't translated.
 */
export function friendlyMessage(err: ApiError): string {
  switch (err.code as ApiErrorCode) {
    case 'auth.unauthorized':
      return 'Please log in to continue.';
    case 'auth.invalid_credentials':
      return 'Email or password is incorrect.';
    case 'auth.not_verified':
      return 'Please verify your email address before continuing.';
    case 'auth.token_invalid':
      return 'This link is invalid or has expired.';
    case 'auth.forbidden':
      return 'You are not allowed to do that.';
    case 'request.invalid_email':
      return 'That email address does not look valid.';
    case 'request.password_too_short':
      return 'Choose a password of at least 12 characters.';
    case 'request.display_name_required':
      return 'Please provide a display name.';
    case 'user.email_taken':
      return 'An account already exists for that email.';
    case 'topic.slug_taken':
      return 'A topic with that slug already exists.';
    case 'delegation.cycle':
      return 'That delegation would create a cycle and was rejected.';
    case 'delegation.self':
      return 'You cannot delegate to yourself.';
    case 'delegation.already_active':
      return 'You already have an active delegation on this topic.';
    case 'proposal.invalid_transition':
      return 'That proposal status change is not allowed.';
    case 'vote.outside_window':
      return 'This proposal is not currently open for voting.';
    case 'vote.proposal_not_in_voting':
      return 'This proposal is not in the voting phase.';
    case 'rate_limited':
      return 'You are doing that too quickly. Please slow down.';
    case 'not_found':
      return 'Not found.';
    default:
      return err.message;
  }
}
