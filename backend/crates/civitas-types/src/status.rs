//! Proposal lifecycle status.

use serde::{Deserialize, Serialize};

/// Where a proposal is in its lifecycle.
///
/// The transitions form a forward-only state machine:
/// `Draft → Deliberation → Voting → Closed`.
/// Backwards transitions are not permitted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "ts-export", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-export", ts(export))]
#[cfg_attr(feature = "sqlx-impl", derive(sqlx::Type))]
#[cfg_attr(
    feature = "sqlx-impl",
    sqlx(type_name = "proposal_status", rename_all = "lowercase")
)]
#[serde(rename_all = "lowercase")]
pub enum ProposalStatus {
    Draft,
    Deliberation,
    Voting,
    Closed,
}

impl ProposalStatus {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            ProposalStatus::Draft => "draft",
            ProposalStatus::Deliberation => "deliberation",
            ProposalStatus::Voting => "voting",
            ProposalStatus::Closed => "closed",
        }
    }

    /// Returns the (single) legal next state, or `None` if terminal.
    #[must_use]
    pub const fn next(self) -> Option<ProposalStatus> {
        match self {
            ProposalStatus::Draft => Some(ProposalStatus::Deliberation),
            ProposalStatus::Deliberation => Some(ProposalStatus::Voting),
            ProposalStatus::Voting => Some(ProposalStatus::Closed),
            ProposalStatus::Closed => None,
        }
    }

    /// Whether a transition `self → target` is legal.
    #[must_use]
    pub fn can_transition_to(self, target: ProposalStatus) -> bool {
        matches!(self.next(), Some(n) if n == target)
    }

    /// True for any state in which votes can be accepted.
    #[must_use]
    pub const fn accepts_votes(self) -> bool {
        matches!(self, ProposalStatus::Voting)
    }

    /// True for any state in which deliberation comments can be posted.
    #[must_use]
    pub const fn accepts_comments(self) -> bool {
        matches!(self, ProposalStatus::Deliberation | ProposalStatus::Voting)
    }
}

impl std::fmt::Display for ProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transitions_are_forward_only() {
        assert!(ProposalStatus::Draft.can_transition_to(ProposalStatus::Deliberation));
        assert!(ProposalStatus::Deliberation.can_transition_to(ProposalStatus::Voting));
        assert!(ProposalStatus::Voting.can_transition_to(ProposalStatus::Closed));

        // No skipping
        assert!(!ProposalStatus::Draft.can_transition_to(ProposalStatus::Voting));
        // No going backwards
        assert!(!ProposalStatus::Voting.can_transition_to(ProposalStatus::Deliberation));
        assert!(!ProposalStatus::Closed.can_transition_to(ProposalStatus::Voting));
    }

    #[test]
    fn closed_is_terminal() {
        assert_eq!(ProposalStatus::Closed.next(), None);
    }

    #[test]
    fn accepts_votes_only_during_voting() {
        assert!(!ProposalStatus::Draft.accepts_votes());
        assert!(!ProposalStatus::Deliberation.accepts_votes());
        assert!(ProposalStatus::Voting.accepts_votes());
        assert!(!ProposalStatus::Closed.accepts_votes());
    }

    #[test]
    fn accepts_comments_during_deliberation_and_voting() {
        assert!(!ProposalStatus::Draft.accepts_comments());
        assert!(ProposalStatus::Deliberation.accepts_comments());
        assert!(ProposalStatus::Voting.accepts_comments());
        assert!(!ProposalStatus::Closed.accepts_comments());
    }
}
