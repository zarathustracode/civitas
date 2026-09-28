//! Change notifications for live tallies.
//!
//! Every write that can move a proposal's tally announces it on the
//! Postgres channel [`CHANNEL`] inside the write's own transaction. Postgres
//! delivers a `NOTIFY` only when that transaction commits, and to every
//! listening connection — so each API process sees every change, whichever
//! process served the write.

use std::fmt;

use sqlx::PgExecutor;

use civitas_types::{ProposalId, TopicId};

use crate::DbResult;

/// The `LISTEN`/`NOTIFY` channel carrying [`TallyScope`] payloads.
pub const CHANNEL: &str = "civitas_tally";

/// Which tallies a committed write may have changed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TallyScope {
    /// A vote was cast on this proposal, or its status changed.
    Proposal(ProposalId),
    /// A delegation on this topic was created or revoked.
    Topic(TopicId),
    /// The eligible-voter set changed, which touches every tally.
    All,
}

impl TallyScope {
    /// Parse a notification payload written by [`notify`].
    #[must_use]
    pub fn parse(payload: &str) -> Option<Self> {
        if payload == "all" {
            return Some(Self::All);
        }
        let (kind, id) = payload.split_once(':')?;
        match kind {
            "proposal" => id.parse().ok().map(Self::Proposal),
            "topic" => id.parse().ok().map(Self::Topic),
            _ => None,
        }
    }
}

impl fmt::Display for TallyScope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Proposal(id) => write!(f, "proposal:{id}"),
            Self::Topic(id) => write!(f, "topic:{id}"),
            Self::All => f.write_str("all"),
        }
    }
}

/// Queue a notification for `scope`. Call it inside the transaction that
/// makes the change so listeners hear about committed writes only.
pub async fn notify<'c, E: PgExecutor<'c>>(conn: E, scope: TallyScope) -> DbResult<()> {
    // Runtime-checked: `pg_notify` returns `void`, which the compile-time
    // `query!` macro cannot describe. `execute` never decodes the row.
    sqlx::query("select pg_notify($1, $2)")
        .bind(CHANNEL)
        .bind(scope.to_string())
        .execute(conn)
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn payloads_round_trip() {
        for scope in [
            TallyScope::Proposal(ProposalId::new()),
            TallyScope::Topic(TopicId::new()),
            TallyScope::All,
        ] {
            assert_eq!(TallyScope::parse(&scope.to_string()), Some(scope));
        }
    }

    #[test]
    fn malformed_payloads_are_rejected() {
        for payload in [
            "",
            "everything",
            "proposal:",
            "proposal:nope",
            "vote:1",
            "topic",
        ] {
            assert_eq!(TallyScope::parse(payload), None, "{payload:?}");
        }
    }
}
