//! Delegation cycle detection.
//!
//! Cycles in a per-topic delegation graph would make tallying undefined.
//! We reject them at delegation **creation time** by walking forward from
//! the proposed delegate; if we reach the proposed delegator, the new edge
//! would close a cycle.
//!
//! Together with the database-level partial unique constraint
//! `UNIQUE (delegator_id, topic_id) WHERE revoked_at IS NULL`, this gives:
//! at most one outgoing edge per `(delegator, topic)`, and the graph is
//! always acyclic at write time.
//!
//! The walk is bounded by [`crate::MAX_DELEGATION_DEPTH`] as a safety fuse
//! against corrupted input. Reaching the fuse is treated as `DepthExceeded`
//! and rejected — better to refuse a legitimate but absurdly deep request
//! than to risk wedging the server.

use std::collections::HashMap;

use civitas_types::{TopicId, UserId};

use crate::records::DelegationRecord;
use crate::MAX_DELEGATION_DEPTH;

/// A delegation a caller would like to create. The caller has not yet
/// inserted any rows; this is the input to the cycle check.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProposedDelegation {
    pub delegator_id: UserId,
    pub delegate_id: UserId,
    pub topic_id: TopicId,
}

/// Result of a cycle check.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CycleCheck {
    /// The new edge does not close a cycle.
    Acyclic,
    /// The new edge would close a cycle.
    Cyclic,
    /// `delegator_id == delegate_id` — never legal.
    SelfDelegation,
    /// The walk hit the depth fuse before resolving. Treat as cyclic.
    DepthExceeded,
}

impl CycleCheck {
    /// True only for [`CycleCheck::Acyclic`].
    #[must_use]
    pub fn is_ok(self) -> bool {
        matches!(self, CycleCheck::Acyclic)
    }
}

/// Pure cycle check.
///
/// `existing` may contain delegations on any topic and any state; the
/// function filters to active delegations on `proposed.topic_id` itself.
#[must_use]
pub fn would_create_cycle(
    existing: &[DelegationRecord],
    proposed: &ProposedDelegation,
) -> CycleCheck {
    if proposed.delegator_id == proposed.delegate_id {
        return CycleCheck::SelfDelegation;
    }

    // Build the per-topic delegate-of map from active delegations.
    let mut delegate_of: HashMap<UserId, UserId> = HashMap::new();
    for d in existing {
        if d.is_active() && d.topic_id == proposed.topic_id {
            // Partial unique at the DB layer makes this insert idempotent
            // for legitimate data; if duplicates show up here we just
            // overwrite — the result is still correct because the walk
            // does not depend on which copy was kept.
            delegate_of.insert(d.delegator_id, d.delegate_id);
        }
    }

    // Walk forward from the proposed delegate. If we reach the proposed
    // delegator, adding the edge `delegator -> delegate` would close a cycle.
    let mut current = proposed.delegate_id;
    for _ in 0..MAX_DELEGATION_DEPTH {
        if current == proposed.delegator_id {
            return CycleCheck::Cyclic;
        }
        match delegate_of.get(&current).copied() {
            Some(next) => current = next,
            None => return CycleCheck::Acyclic,
        }
    }

    CycleCheck::DepthExceeded
}

#[cfg(test)]
#[allow(clippy::many_single_char_names)] // a, b, c, … are users; the chain shape is the point.
mod tests {
    use super::*;
    use chrono::Utc;
    use civitas_types::TopicId;
    use pretty_assertions::assert_eq;

    fn topic() -> TopicId {
        TopicId::new()
    }

    fn fresh_users(n: usize) -> Vec<UserId> {
        (0..n).map(|_| UserId::new()).collect()
    }

    #[test]
    fn self_delegation_rejected() {
        let u = UserId::new();
        let t = topic();
        let r = would_create_cycle(
            &[],
            &ProposedDelegation {
                delegator_id: u,
                delegate_id: u,
                topic_id: t,
            },
        );
        assert_eq!(r, CycleCheck::SelfDelegation);
    }

    #[test]
    fn empty_graph_is_acyclic() {
        let users = fresh_users(2);
        let r = would_create_cycle(
            &[],
            &ProposedDelegation {
                delegator_id: users[0],
                delegate_id: users[1],
                topic_id: topic(),
            },
        );
        assert_eq!(r, CycleCheck::Acyclic);
    }

    #[test]
    fn simple_two_node_cycle_rejected() {
        // A -> B exists. Trying to add B -> A on the same topic creates a cycle.
        let t = topic();
        let users = fresh_users(2);
        let a = users[0];
        let b = users[1];
        let existing = vec![DelegationRecord::active(a, b, t, Utc::now())];

        let r = would_create_cycle(
            &existing,
            &ProposedDelegation {
                delegator_id: b,
                delegate_id: a,
                topic_id: t,
            },
        );
        assert_eq!(r, CycleCheck::Cyclic);
    }

    #[test]
    fn three_node_cycle_rejected() {
        // A -> B -> C exists. Adding C -> A closes the loop.
        let t = topic();
        let users = fresh_users(3);
        let (a, b, c) = (users[0], users[1], users[2]);
        let existing = vec![
            DelegationRecord::active(a, b, t, Utc::now()),
            DelegationRecord::active(b, c, t, Utc::now()),
        ];

        let r = would_create_cycle(
            &existing,
            &ProposedDelegation {
                delegator_id: c,
                delegate_id: a,
                topic_id: t,
            },
        );
        assert_eq!(r, CycleCheck::Cyclic);
    }

    #[test]
    fn long_chain_then_close_rejected() {
        // A -> B -> C -> D -> E -> F. Adding F -> A closes a 6-cycle.
        let t = topic();
        let users = fresh_users(6);
        let mut existing = Vec::new();
        for i in 0..(users.len() - 1) {
            existing.push(DelegationRecord::active(
                users[i],
                users[i + 1],
                t,
                Utc::now(),
            ));
        }

        let r = would_create_cycle(
            &existing,
            &ProposedDelegation {
                delegator_id: users[5],
                delegate_id: users[0],
                topic_id: t,
            },
        );
        assert_eq!(r, CycleCheck::Cyclic);
    }

    #[test]
    fn parallel_branch_does_not_falsely_match() {
        // A -> B exists; an unrelated C -> D delegation also exists.
        // Adding D -> E (a leaf) is acyclic.
        let t = topic();
        let users = fresh_users(5);
        let (a, b, c, d, e) = (users[0], users[1], users[2], users[3], users[4]);
        let existing = vec![
            DelegationRecord::active(a, b, t, Utc::now()),
            DelegationRecord::active(c, d, t, Utc::now()),
        ];

        let r = would_create_cycle(
            &existing,
            &ProposedDelegation {
                delegator_id: d,
                delegate_id: e,
                topic_id: t,
            },
        );
        assert_eq!(r, CycleCheck::Acyclic);
    }

    #[test]
    fn cycle_on_other_topic_does_not_block() {
        // A -> B on topic X. Adding B -> A on topic Y is fine.
        let t1 = topic();
        let t2 = topic();
        let users = fresh_users(2);
        let (a, b) = (users[0], users[1]);
        let existing = vec![DelegationRecord::active(a, b, t1, Utc::now())];

        let r = would_create_cycle(
            &existing,
            &ProposedDelegation {
                delegator_id: b,
                delegate_id: a,
                topic_id: t2,
            },
        );
        assert_eq!(r, CycleCheck::Acyclic);
    }

    #[test]
    fn revoked_delegation_is_ignored() {
        // A -> B was revoked. Attempting B -> A is acyclic.
        let t = topic();
        let users = fresh_users(2);
        let (a, b) = (users[0], users[1]);
        let mut revoked = DelegationRecord::active(a, b, t, Utc::now());
        revoked.revoked_at = Some(Utc::now());

        let r = would_create_cycle(
            &[revoked],
            &ProposedDelegation {
                delegator_id: b,
                delegate_id: a,
                topic_id: t,
            },
        );
        assert_eq!(r, CycleCheck::Acyclic);
    }

    #[test]
    fn dangling_chain_is_acyclic() {
        // A -> B -> C, no further edges. Adding D -> A is acyclic.
        let t = topic();
        let users = fresh_users(4);
        let (a, b, c, d) = (users[0], users[1], users[2], users[3]);
        let existing = vec![
            DelegationRecord::active(a, b, t, Utc::now()),
            DelegationRecord::active(b, c, t, Utc::now()),
        ];

        let r = would_create_cycle(
            &existing,
            &ProposedDelegation {
                delegator_id: d,
                delegate_id: a,
                topic_id: t,
            },
        );
        assert_eq!(r, CycleCheck::Acyclic);
    }
}
