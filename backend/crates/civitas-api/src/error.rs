//! HTTP error responses.
//!
//! Every error the API returns has a stable `code` (machine-readable) and a
//! human-readable `message`. The `code` is the contract between server and
//! frontend; `message` is for fallback display only and may evolve.
//!
//! Internal error details (SQL strings, panics, internal addresses) are
//! never exposed in the response body. They appear in `tracing` logs.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;
use tracing::error;

use civitas_auth::AuthError;
use civitas_db::DbError;

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    // ── Authentication / authorization ──
    #[error("authentication required")]
    Unauthorized,
    #[error("invalid credentials")]
    InvalidCredentials,
    #[error("account email is not verified")]
    NotVerified,
    #[error("token invalid or expired")]
    TokenInvalid,
    #[error("forbidden")]
    Forbidden,

    // ── Validation ──
    #[error("invalid request: {0}")]
    BadRequest(String),
    #[error("invalid email")]
    InvalidEmail,
    #[error("password too short (minimum 12 characters)")]
    PasswordTooShort,
    #[error("display name is required")]
    DisplayNameRequired,
    #[error("a non-empty reason is required")]
    ReasonRequired,

    // ── Conflicts ──
    #[error("email already taken")]
    EmailAlreadyTaken,
    #[error("phone already taken")]
    PhoneAlreadyTaken,
    #[error("topic slug already taken")]
    SlugAlreadyTaken,
    #[error("delegation would close a cycle")]
    DelegationCycle,
    #[error("delegator and delegate must differ")]
    DelegationSelf,
    #[error("user already has an active delegation on this topic")]
    DelegationAlreadyActive,

    // ── Domain rules ──
    #[error("not found")]
    NotFound,
    #[error("invalid state transition: {from} → {to}")]
    InvalidStateTransition {
        from: &'static str,
        to: &'static str,
    },
    #[error("voting window is required when entering voting status")]
    VotingWindowRequired,
    #[error("voting window must have start < end")]
    VotingWindowInvalid,
    #[error("vote outside the proposal's voting window")]
    VoteOutsideWindow,
    #[error("proposal is not in voting status")]
    ProposalNotInVoting,
    #[error("comments are not allowed in {0}")]
    CommentsNotAllowedInStatus(&'static str),
    #[error("parent comment belongs to a different proposal")]
    CommentParentMismatch,

    // ── Rate limiting ──
    #[error("rate limited")]
    RateLimited,

    // ── Internal ──
    #[error("internal error")]
    Internal(#[source] anyhow::Error),
}

#[derive(Debug, Serialize)]
struct ErrorBody<'a> {
    error: ErrorPayload<'a>,
}

#[derive(Debug, Serialize)]
struct ErrorPayload<'a> {
    code: &'a str,
    message: &'a str,
}

impl ApiError {
    fn status_and_code(&self) -> (StatusCode, &'static str) {
        match self {
            ApiError::Unauthorized => (StatusCode::UNAUTHORIZED, "auth.unauthorized"),
            ApiError::InvalidCredentials => (StatusCode::UNAUTHORIZED, "auth.invalid_credentials"),
            ApiError::NotVerified => (StatusCode::FORBIDDEN, "auth.not_verified"),
            ApiError::TokenInvalid => (StatusCode::BAD_REQUEST, "auth.token_invalid"),
            ApiError::Forbidden => (StatusCode::FORBIDDEN, "auth.forbidden"),

            ApiError::BadRequest(_) => (StatusCode::BAD_REQUEST, "request.bad"),
            ApiError::InvalidEmail => (StatusCode::BAD_REQUEST, "request.invalid_email"),
            ApiError::PasswordTooShort => (StatusCode::BAD_REQUEST, "request.password_too_short"),
            ApiError::DisplayNameRequired => {
                (StatusCode::BAD_REQUEST, "request.display_name_required")
            }
            ApiError::ReasonRequired => (StatusCode::BAD_REQUEST, "request.reason_required"),

            ApiError::EmailAlreadyTaken => (StatusCode::CONFLICT, "user.email_taken"),
            ApiError::PhoneAlreadyTaken => (StatusCode::CONFLICT, "user.phone_taken"),
            ApiError::SlugAlreadyTaken => (StatusCode::CONFLICT, "topic.slug_taken"),
            ApiError::DelegationCycle => (StatusCode::CONFLICT, "delegation.cycle"),
            ApiError::DelegationSelf => (StatusCode::BAD_REQUEST, "delegation.self"),
            ApiError::DelegationAlreadyActive => {
                (StatusCode::CONFLICT, "delegation.already_active")
            }

            ApiError::NotFound => (StatusCode::NOT_FOUND, "not_found"),
            ApiError::InvalidStateTransition { .. } => {
                (StatusCode::CONFLICT, "proposal.invalid_transition")
            }
            ApiError::VotingWindowRequired => {
                (StatusCode::BAD_REQUEST, "proposal.voting_window_required")
            }
            ApiError::VotingWindowInvalid => {
                (StatusCode::BAD_REQUEST, "proposal.voting_window_invalid")
            }
            ApiError::VoteOutsideWindow => (StatusCode::CONFLICT, "vote.outside_window"),
            ApiError::ProposalNotInVoting => (StatusCode::CONFLICT, "vote.proposal_not_in_voting"),
            ApiError::CommentsNotAllowedInStatus(_) => {
                (StatusCode::CONFLICT, "comment.not_allowed_in_status")
            }
            ApiError::CommentParentMismatch => (StatusCode::BAD_REQUEST, "comment.parent_mismatch"),

            ApiError::RateLimited => (StatusCode::TOO_MANY_REQUESTS, "rate_limited"),
            ApiError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, "internal"),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, code) = self.status_and_code();
        let message = self.to_string();

        // Internal errors get logged but never expose detail to the client.
        if let ApiError::Internal(ref inner) = self {
            error!(error = ?inner, "internal api error");
        }

        let body = ErrorBody {
            error: ErrorPayload {
                code,
                message: &message,
            },
        };
        (status, Json(body)).into_response()
    }
}

// ── conversions from lower-layer errors ────────────────────────────────────

impl From<AuthError> for ApiError {
    fn from(err: AuthError) -> Self {
        match err {
            AuthError::InvalidCredentials => ApiError::InvalidCredentials,
            AuthError::NotVerified => ApiError::NotVerified,
            AuthError::TokenExpired | AuthError::TokenInvalid => ApiError::TokenInvalid,
            AuthError::RateLimited => ApiError::RateLimited,
            AuthError::InvalidEmail => ApiError::InvalidEmail,
            AuthError::PasswordTooShort => ApiError::PasswordTooShort,
            AuthError::DisplayNameRequired => ApiError::DisplayNameRequired,
            AuthError::EmailAlreadyTaken => ApiError::EmailAlreadyTaken,
            AuthError::Db(e) => ApiError::from(e),
            AuthError::Sqlx(e) => ApiError::Internal(anyhow::anyhow!(e)),
            AuthError::Internal(msg) => ApiError::Internal(anyhow::anyhow!(msg)),
        }
    }
}

impl From<DbError> for ApiError {
    fn from(err: DbError) -> Self {
        match err {
            DbError::NotFound => ApiError::NotFound,
            DbError::EmailAlreadyTaken => ApiError::EmailAlreadyTaken,
            DbError::PhoneAlreadyTaken => ApiError::PhoneAlreadyTaken,
            DbError::SlugAlreadyTaken => ApiError::SlugAlreadyTaken,
            DbError::InvalidStateTransition { from, to } => {
                ApiError::InvalidStateTransition { from, to }
            }
            DbError::VotingWindowRequired => ApiError::VotingWindowRequired,
            DbError::VotingWindowInvalid => ApiError::VotingWindowInvalid,
            DbError::ProposalNotInVoting => ApiError::ProposalNotInVoting,
            DbError::OutsideVotingWindow => ApiError::VoteOutsideWindow,
            DbError::DelegationSelf => ApiError::DelegationSelf,
            DbError::DelegationCyclic | DbError::DelegationDepthExceeded => {
                ApiError::DelegationCycle
            }
            DbError::DelegationAlreadyActive => ApiError::DelegationAlreadyActive,
            DbError::CommentsNotAllowedInStatus(s) => ApiError::CommentsNotAllowedInStatus(s),
            DbError::CommentParentMismatch => ApiError::CommentParentMismatch,
            DbError::ReasonRequired => ApiError::ReasonRequired,
            DbError::Sqlx(e) => ApiError::Internal(anyhow::anyhow!(e)),
            DbError::Migrate(e) => ApiError::Internal(anyhow::anyhow!(e)),
        }
    }
}

impl From<sqlx::Error> for ApiError {
    fn from(err: sqlx::Error) -> Self {
        ApiError::Internal(anyhow::anyhow!(err))
    }
}

pub type ApiResult<T> = std::result::Result<T, ApiError>;
