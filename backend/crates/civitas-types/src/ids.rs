//! Newtype-wrapped UUIDs for every domain entity.
//!
//! The compiler refuses to mix them — passing a `UserId` where a `ProposalId`
//! is expected fails to compile. `UUIDv7` is used so generated rows are
//! roughly time-sortable at the storage layer.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

macro_rules! id_newtype {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
        #[cfg_attr(feature = "ts-export", derive(ts_rs::TS))]
        #[cfg_attr(feature = "ts-export", ts(export, type = "string"))]
        #[cfg_attr(feature = "sqlx-impl", derive(sqlx::Type))]
        #[cfg_attr(feature = "sqlx-impl", sqlx(transparent))]
        #[serde(transparent)]
        pub struct $name(pub Uuid);

        impl $name {
            /// Generate a fresh UUIDv7-backed identifier.
            #[must_use]
            pub fn new() -> Self {
                Self(Uuid::now_v7())
            }

            /// Wrap an existing UUID.
            #[must_use]
            pub const fn from_uuid(u: Uuid) -> Self {
                Self(u)
            }

            /// Unwrap to the underlying UUID.
            #[must_use]
            pub const fn into_inner(self) -> Uuid {
                self.0
            }

            /// Borrow the underlying UUID.
            #[must_use]
            pub const fn as_uuid(&self) -> &Uuid {
                &self.0
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.fmt(f)
            }
        }

        impl FromStr for $name {
            type Err = uuid::Error;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Uuid::from_str(s).map(Self)
            }
        }

        impl From<Uuid> for $name {
            fn from(u: Uuid) -> Self {
                Self(u)
            }
        }

        impl From<$name> for Uuid {
            fn from(id: $name) -> Self {
                id.0
            }
        }
    };
}

id_newtype!(
    /// Stable identifier for a user account.
    UserId
);
id_newtype!(
    /// Stable identifier for a topic.
    TopicId
);
id_newtype!(
    /// Stable identifier for a proposal.
    ProposalId
);
id_newtype!(
    /// Stable identifier for a single vote-cast event (append-only log row).
    VoteId
);
id_newtype!(
    /// Stable identifier for a delegation row.
    DelegationId
);
id_newtype!(
    /// Stable identifier for a deliberation comment.
    CommentId
);
id_newtype!(
    /// Stable identifier for an audit-log event.
    AuditLogId
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distinct_types_do_not_mix() {
        // This is a compile-time guarantee, exercised here for clarity.
        let u = UserId::new();
        let p = ProposalId::new();
        assert_ne!(u.into_inner(), p.into_inner());
    }

    #[test]
    fn round_trip_string() {
        let u = UserId::new();
        let s = u.to_string();
        let parsed: UserId = s.parse().unwrap();
        assert_eq!(u, parsed);
    }

    #[test]
    fn uuid_v7_is_monotonic_in_practice() {
        // UUIDv7 embeds a millisecond timestamp; two IDs created back to back
        // should compare in creation order under lexical / Ord comparison.
        let a = UserId::new();
        // Tight loop is fine; we just need a guaranteed clock tick on most platforms.
        std::thread::sleep(std::time::Duration::from_millis(2));
        let b = UserId::new();
        assert!(a < b, "UUIDv7 should be roughly time-ordered: {a} vs {b}");
    }
}
