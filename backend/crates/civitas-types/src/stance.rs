//! Deliberation comment stance.

use serde::{Deserialize, Serialize};

/// Where the author of a deliberation comment stands on the proposal.
///
/// Required at comment creation. The structural label keeps deliberation
/// threads legible: "12 in support, 4 opposed, 7 neutral, 3 questions" is
/// derivable cheaply at the top of every thread.
///
/// Independent of the author's actual vote — a `Support` comment can
/// precede a `No` vote, and that is allowed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "ts-export", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-export", ts(export))]
#[serde(rename_all = "lowercase")]
pub enum Stance {
    Support,
    Oppose,
    Neutral,
    Question,
}

impl Stance {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Stance::Support => "support",
            Stance::Oppose => "oppose",
            Stance::Neutral => "neutral",
            Stance::Question => "question",
        }
    }
}

impl std::fmt::Display for Stance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        for s in [
            Stance::Support,
            Stance::Oppose,
            Stance::Neutral,
            Stance::Question,
        ] {
            let json = serde_json::to_string(&s).unwrap();
            let back: Stance = serde_json::from_str(&json).unwrap();
            assert_eq!(s, back);
        }
    }
}
