//! Input records for the pure tally and delegation algorithms.
//!
//! These are plain data structures. The database layer constructs them
//! from query rows; the tally function consumes them. They are also
//! serializable for tests and replay.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use civitas_types::{DelegationId, ProposalId, TopicId, UserId, VoteChoice, VoteId, Weight};

/// A single recorded vote-cast event.
///
/// Vote rows are append-only. To "change" a vote during a voting window the
/// user inserts a new row. The tally algorithm assumes its input has already
/// been filtered to the **active** vote per `(proposal_id, voter_id)` —
/// callers are responsible for that filter.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VoteRecord {
    pub id: VoteId,
    pub proposal_id: ProposalId,
    pub voter_id: UserId,
    pub choice: VoteChoice,
    pub cast_at: DateTime<Utc>,
}

impl VoteRecord {
    /// Convenience constructor for tests and seed data.
    #[must_use]
    pub fn new(
        proposal_id: ProposalId,
        voter_id: UserId,
        choice: VoteChoice,
        cast_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id: VoteId::new(),
            proposal_id,
            voter_id,
            choice,
            cast_at,
        }
    }
}

/// A delegation row.
///
/// Active delegations have `revoked_at == None`. Tally and cycle-check
/// helpers expect callers to filter to active rows where appropriate, but
/// they will also tolerate revoked rows being mixed in (and ignore them).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DelegationRecord {
    pub id: DelegationId,
    pub delegator_id: UserId,
    pub delegate_id: UserId,
    pub topic_id: TopicId,
    pub created_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
}

impl DelegationRecord {
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.revoked_at.is_none()
    }

    /// Convenience constructor for tests.
    #[must_use]
    pub fn active(
        delegator_id: UserId,
        delegate_id: UserId,
        topic_id: TopicId,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id: DelegationId::new(),
            delegator_id,
            delegate_id,
            topic_id,
            created_at,
            revoked_at: None,
        }
    }
}

/// A user who is eligible to vote on the proposal under tally, paired with
/// their base weight. The tally function iterates this list — anyone not in
/// it does not contribute, regardless of votes or delegations recorded for
/// them. This mirrors the policy decision at the API layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct EligibleUser {
    pub user_id: UserId,
    pub weight: Weight,
}

impl EligibleUser {
    /// One-person-one-vote convenience: weight = 1.
    #[must_use]
    pub fn unit(user_id: UserId) -> Self {
        Self {
            user_id,
            weight: Weight::ONE,
        }
    }
}
