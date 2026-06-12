//! Request and response shapes shared between routes.
//!
//! Keep these stable — they are the API contract. Internal storage shapes
//! belong in `civitas-db`; these are the wire types.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use civitas_types::{
    CommentId, DelegationId, ProposalId, ProposalStatus, Stance, TopicId, UserId, VoteChoice,
    VoteId, Weight,
};

// ── users ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct UserResponse {
    pub id: UserId,
    pub email: String,
    pub display_name: String,
    pub email_verified: bool,
    pub phone_verified: bool,
    pub created_at: DateTime<Utc>,
}

impl From<civitas_db::users::User> for UserResponse {
    fn from(u: civitas_db::users::User) -> Self {
        Self {
            id: u.id,
            email: u.email,
            display_name: u.display_name,
            email_verified: u.email_verified_at.is_some(),
            phone_verified: u.phone_verified_at.is_some(),
            created_at: u.created_at,
        }
    }
}

// ── topics ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct TopicResponse {
    pub id: TopicId,
    pub slug: String,
    pub name: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
}

impl From<civitas_db::topics::Topic> for TopicResponse {
    fn from(t: civitas_db::topics::Topic) -> Self {
        Self {
            id: t.id,
            slug: t.slug,
            name: t.name,
            description: t.description,
            created_at: t.created_at,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateTopicRequest {
    pub slug: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct ProposalCounts {
    pub draft: i64,
    pub deliberation: i64,
    pub voting: i64,
    pub closed: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct TopDelegate {
    pub id: UserId,
    pub display_name: String,
    pub incoming: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct TopicStatsResponse {
    pub topic_id: TopicId,
    pub proposal_counts: ProposalCounts,
    pub active_delegations: i64,
    pub top_delegates: Vec<TopDelegate>,
}

// ── proposals ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct ProposalResponse {
    pub id: ProposalId,
    pub topic_id: TopicId,
    pub title: String,
    pub summary: String,
    pub body: String,
    pub author_id: UserId,
    pub status: ProposalStatus,
    pub voting_starts_at: Option<DateTime<Utc>>,
    pub voting_ends_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<civitas_db::proposals::Proposal> for ProposalResponse {
    fn from(p: civitas_db::proposals::Proposal) -> Self {
        Self {
            id: p.id,
            topic_id: p.topic_id,
            title: p.title,
            summary: p.summary,
            body: p.body,
            author_id: p.author_id,
            status: p.status,
            voting_starts_at: p.voting_starts_at,
            voting_ends_at: p.voting_ends_at,
            created_at: p.created_at,
            updated_at: p.updated_at,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateProposalRequest {
    pub topic_id: TopicId,
    pub title: String,
    pub summary: String,
    pub body: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TransitionStatusRequest {
    pub target: ProposalStatus,
    pub voting_starts_at: Option<DateTime<Utc>>,
    pub voting_ends_at: Option<DateTime<Utc>>,
}

// ── votes / tally ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct CastVoteRequest {
    pub choice: VoteChoice,
}

#[derive(Debug, Clone, Serialize)]
pub struct VoteResponse {
    pub id: VoteId,
    pub proposal_id: ProposalId,
    pub voter_id: UserId,
    pub choice: VoteChoice,
    pub cast_at: DateTime<Utc>,
}

impl From<civitas_db::votes::VoteRow> for VoteResponse {
    fn from(v: civitas_db::votes::VoteRow) -> Self {
        Self {
            id: v.id,
            proposal_id: v.proposal_id,
            voter_id: v.voter_id,
            choice: v.choice,
            cast_at: v.cast_at,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct TallyResponse {
    pub proposal_id: ProposalId,
    pub yes: Weight,
    pub no: Weight,
    pub abstain: Weight,
    pub eligible_voters: usize,
    pub counted_voters: usize,
    /// Trail entry for the requesting user, with delegate names resolved.
    /// `None` for anonymous viewers or users not in the eligible set.
    pub your_trail: Option<UserTrail>,
}

/// How the requesting user's weight reaches a vote, with display names.
/// Mirrors `civitas_core::tally::TrailKind` but resolves UUIDs to names so
/// the frontend can render without a second round-trip.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum UserTrail {
    /// You voted directly.
    Direct { choice: VoteChoice },
    /// Your weight flowed through a delegation chain to `terminal`.
    /// `path` is the intermediate hops (display names), excluding both
    /// you and the terminal voter.
    Delegated {
        path: Vec<NamedUser>,
        terminal: NamedUser,
        choice: VoteChoice,
    },
    /// Your weight was not counted.
    NotCounted { reason: String },
}

#[derive(Debug, Clone, Serialize)]
pub struct NamedUser {
    pub id: UserId,
    pub display_name: String,
}

// ── delegations ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct DelegationResponse {
    pub id: DelegationId,
    pub delegator_id: UserId,
    pub delegate_id: UserId,
    /// Display name of the delegate. `None` only when the row could not be
    /// joined (e.g. legacy soft-deleted user); UI should fall back to the id.
    pub delegate_display_name: Option<String>,
    pub topic_id: TopicId,
    pub created_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
}

impl From<civitas_db::delegations::DelegationRow> for DelegationResponse {
    fn from(d: civitas_db::delegations::DelegationRow) -> Self {
        Self {
            id: d.id,
            delegator_id: d.delegator_id,
            delegate_id: d.delegate_id,
            delegate_display_name: None,
            topic_id: d.topic_id,
            created_at: d.created_at,
            revoked_at: d.revoked_at,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateDelegationRequest {
    pub topic_id: TopicId,
    pub delegate_id: UserId,
}

// ── comments ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct CommentResponse {
    pub id: CommentId,
    pub proposal_id: ProposalId,
    pub author_id: UserId,
    pub parent_id: Option<CommentId>,
    pub body: String,
    pub stance: Stance,
    pub created_at: DateTime<Utc>,
    pub edited_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub hidden_at: Option<DateTime<Utc>>,
    pub hidden_reason: Option<String>,
}

impl From<civitas_db::comments::CommentRow> for CommentResponse {
    fn from(c: civitas_db::comments::CommentRow) -> Self {
        Self {
            id: c.id,
            proposal_id: c.proposal_id,
            author_id: c.author_id,
            parent_id: c.parent_id,
            body: c.body,
            stance: c.stance,
            created_at: c.created_at,
            edited_at: c.edited_at,
            deleted_at: c.deleted_at,
            hidden_at: c.hidden_at,
            hidden_reason: c.hidden_reason,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateCommentRequest {
    pub parent_id: Option<CommentId>,
    pub body: String,
    pub stance: Stance,
}

// ── audit log ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct AuditEntryResponse {
    pub id: civitas_types::AuditLogId,
    /// Actor's display name, or `null` for system-initiated events
    /// (auto-close, scheduled cleanups) and for actors whose user row was
    /// soft-deleted.
    pub actor_display_name: Option<String>,
    pub action: String,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

// ── auth ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub display_name: String,
}

/// Successful registration. Mirrors `UserResponse` plus an optional
/// `dev_verification_token` populated only when
/// `DEV_RETURN_VERIFICATION_TOKEN=true` — production deployments never
/// set the field.
#[derive(Debug, Clone, Serialize)]
pub struct RegisterResponse {
    pub id: UserId,
    pub email: String,
    pub display_name: String,
    pub email_verified: bool,
    pub phone_verified: bool,
    pub created_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dev_verification_token: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct VerifyEmailRequest {
    pub token: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ResendVerificationRequest {
    pub email: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PasswordResetRequest {
    pub email: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PasswordResetCompleteRequest {
    pub token: String,
    pub new_password: String,
}
