//! Vote choice — the three options a voter has on any proposal.

use serde::{Deserialize, Serialize};

/// What a voter chose on a proposal.
///
/// `Abstain` is recorded explicitly. It is not the same as not voting —
/// abstain is a deliberate participation that registers presence and
/// neutrality. Tallies report yes / no / abstain separately.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "ts-export", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-export", ts(export))]
#[cfg_attr(feature = "sqlx-impl", derive(sqlx::Type))]
#[cfg_attr(
    feature = "sqlx-impl",
    sqlx(type_name = "vote_choice", rename_all = "lowercase")
)]
#[serde(rename_all = "lowercase")]
pub enum VoteChoice {
    Yes,
    No,
    Abstain,
}

impl VoteChoice {
    /// Stable string representation for storage and logs.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            VoteChoice::Yes => "yes",
            VoteChoice::No => "no",
            VoteChoice::Abstain => "abstain",
        }
    }
}

impl std::fmt::Display for VoteChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_lowercase() {
        assert_eq!(serde_json::to_string(&VoteChoice::Yes).unwrap(), "\"yes\"");
        assert_eq!(serde_json::to_string(&VoteChoice::No).unwrap(), "\"no\"");
        assert_eq!(
            serde_json::to_string(&VoteChoice::Abstain).unwrap(),
            "\"abstain\""
        );
    }

    #[test]
    fn round_trip() {
        for c in [VoteChoice::Yes, VoteChoice::No, VoteChoice::Abstain] {
            let s = serde_json::to_string(&c).unwrap();
            let parsed: VoteChoice = serde_json::from_str(&s).unwrap();
            assert_eq!(c, parsed);
        }
    }
}
