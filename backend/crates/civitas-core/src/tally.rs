//! Pure proposal tallying with delegation chain resolution.
//!
//! This is the load-bearing function. Read it carefully if you are
//! reviewing the project — it answers, given a set of recorded votes and
//! delegations, what the result of a proposal is.
//!
//! ## Inputs
//!
//! 1. `proposal_id` and `proposal_topic` — the proposal under tally.
//! 2. `votes` — the **active** vote per `(proposal_id, voter_id)`. Callers
//!    are responsible for collapsing append-only history to one row per
//!    user (the most recent `cast_at` per user wins). Votes for other
//!    proposals in the slice are filtered out by this function.
//! 3. `delegations` — delegations for the topic. Inactive (revoked) and
//!    other-topic rows are tolerated and ignored.
//! 4. `eligible_users` — the universe of users whose weights count, paired
//!    with their per-user weight. Anyone not in this slice contributes
//!    nothing, even if they appear in `votes`.
//!
//! ## Algorithm
//!
//! For each eligible user U:
//!
//! 1. If U has an active direct vote on the proposal → count U's weight
//!    toward U's choice.
//! 2. Otherwise, walk U's chain of active delegations on the proposal's
//!    topic. If the chain reaches a user X who voted directly, count U's
//!    weight toward X's choice.
//! 3. If the chain ends without a direct voter, U's weight is not counted.
//! 4. If the walk hits the depth fuse (corrupted dataset), U's weight is
//!    not counted.
//!
//! ## Properties
//!
//! - Pure: no I/O, no clock, no randomness. Deterministic.
//! - Direct-vote-dominates: a user's direct vote on a proposal causes
//!   their delegation for that topic to be ignored *for that proposal*.
//! - Per-topic: delegations on other topics are not consulted.
//! - Defensive: cycles or depth-exceeded chains never loop forever; the
//!   user is reported as `NotCounted`.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use civitas_types::{ProposalId, TopicId, UserId, VoteChoice, Weight};

use crate::records::{DelegationRecord, EligibleUser, VoteRecord};
use crate::MAX_DELEGATION_DEPTH;

/// The result of tallying a proposal at a moment in time.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tally {
    pub proposal_id: ProposalId,
    pub yes: Weight,
    pub no: Weight,
    pub abstain: Weight,
    /// Per-eligible-user record of how that user's weight flowed.
    pub trail: Vec<TrailEntry>,
}

impl Tally {
    /// Total weight that contributed to the result.
    #[must_use]
    pub fn counted_weight(&self) -> Weight {
        self.yes + self.no + self.abstain
    }

    /// Total weight of users who did not contribute (chain ended,
    /// depth exceeded, etc).
    #[must_use]
    pub fn uncounted_weight(&self) -> Weight {
        self.trail
            .iter()
            .filter(|e| matches!(e.kind, TrailKind::NotCounted { .. }))
            .map(|e| e.weight)
            .sum()
    }
}

/// One row of the tally trail — what happened to one eligible user's weight.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrailEntry {
    pub user_id: UserId,
    pub weight: Weight,
    pub kind: TrailKind,
}

/// How the user's weight reached (or didn't reach) a vote.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TrailKind {
    /// The user voted directly on this proposal.
    Direct { choice: VoteChoice },
    /// The user did not vote directly; the delegation chain reached
    /// `terminal`, who voted `choice`. `path` lists the *intermediate*
    /// hops between the user and the terminal voter (excluding both).
    Delegated {
        path: Vec<UserId>,
        terminal: UserId,
        choice: VoteChoice,
    },
    /// The user's weight was not counted.
    NotCounted { reason: NotCountedReason },
}

/// Why a user's weight did not contribute.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotCountedReason {
    /// The user did not vote, and their delegation chain ended at a user
    /// who also did not vote.
    NoDirectVoteInChain,
    /// The user's chain walk hit the depth fuse — implies cyclic or
    /// pathologically deep input. Logged as an anomaly upstream.
    DepthExceeded,
}

/// Pure tally function. See module docs for semantics and contract.
#[must_use]
pub fn tally(
    proposal_id: ProposalId,
    proposal_topic: TopicId,
    votes: &[VoteRecord],
    delegations: &[DelegationRecord],
    eligible_users: &[EligibleUser],
) -> Tally {
    // Direct votes for this proposal, keyed by voter_id.
    // (Other-proposal rows are filtered out so the caller can pass a wider slice.)
    let direct: HashMap<UserId, &VoteRecord> = votes
        .iter()
        .filter(|v| v.proposal_id == proposal_id)
        .map(|v| (v.voter_id, v))
        .collect();

    // Per-topic delegate-of map for active delegations.
    let delegate_of: HashMap<UserId, UserId> = delegations
        .iter()
        .filter(|d| d.is_active() && d.topic_id == proposal_topic)
        .map(|d| (d.delegator_id, d.delegate_id))
        .collect();

    let mut yes = Weight::ZERO;
    let mut no = Weight::ZERO;
    let mut abstain = Weight::ZERO;
    let mut trail: Vec<TrailEntry> = Vec::with_capacity(eligible_users.len());

    for u in eligible_users {
        let kind = if let Some(direct_vote) = direct.get(&u.user_id) {
            TrailKind::Direct {
                choice: direct_vote.choice,
            }
        } else {
            resolve(u.user_id, &direct, &delegate_of)
        };

        let counted_choice: Option<VoteChoice> = match &kind {
            TrailKind::Direct { choice } | TrailKind::Delegated { choice, .. } => Some(*choice),
            TrailKind::NotCounted { .. } => None,
        };

        if let Some(c) = counted_choice {
            match c {
                VoteChoice::Yes => yes += u.weight,
                VoteChoice::No => no += u.weight,
                VoteChoice::Abstain => abstain += u.weight,
            }
        }

        trail.push(TrailEntry {
            user_id: u.user_id,
            weight: u.weight,
            kind,
        });
    }

    Tally {
        proposal_id,
        yes,
        no,
        abstain,
        trail,
    }
}

/// Walk the delegation chain from `start`. The starting user is known
/// to *not* have a direct vote (otherwise the caller would have taken
/// the `Direct` branch).
fn resolve(
    start: UserId,
    direct: &HashMap<UserId, &VoteRecord>,
    delegate_of: &HashMap<UserId, UserId>,
) -> TrailKind {
    let mut current = start;
    let mut path: Vec<UserId> = Vec::new();

    for _ in 0..MAX_DELEGATION_DEPTH {
        if let Some(direct_vote) = direct.get(&current) {
            return TrailKind::Delegated {
                path,
                terminal: current,
                choice: direct_vote.choice,
            };
        }

        match delegate_of.get(&current).copied() {
            Some(next) => {
                if current != start {
                    path.push(current);
                }
                current = next;
            }
            None => {
                return TrailKind::NotCounted {
                    reason: NotCountedReason::NoDirectVoteInChain,
                };
            }
        }
    }

    TrailKind::NotCounted {
        reason: NotCountedReason::DepthExceeded,
    }
}

#[cfg(test)]
#[allow(clippy::many_single_char_names)] // a, b, c, … are users; the chain shape is the point.
mod tests {
    use super::*;
    use chrono::{Duration, Utc};
    use pretty_assertions::assert_eq;

    use civitas_types::{ProposalId, TopicId, UserId, VoteChoice, Weight};

    fn vote(prop: ProposalId, voter: UserId, choice: VoteChoice) -> VoteRecord {
        VoteRecord::new(prop, voter, choice, Utc::now())
    }

    fn deleg(d: UserId, to: UserId, t: TopicId) -> DelegationRecord {
        DelegationRecord::active(d, to, t, Utc::now())
    }

    fn revoked_deleg(d: UserId, to: UserId, t: TopicId) -> DelegationRecord {
        let mut r = DelegationRecord::active(d, to, t, Utc::now());
        r.revoked_at = Some(Utc::now());
        r
    }

    fn eligible_unit(users: &[UserId]) -> Vec<EligibleUser> {
        users.iter().copied().map(EligibleUser::unit).collect()
    }

    #[test]
    fn empty_proposal_is_all_zero() {
        let p = ProposalId::new();
        let t = TopicId::new();
        let r = tally(p, t, &[], &[], &[]);
        assert_eq!(r.yes, Weight::ZERO);
        assert_eq!(r.no, Weight::ZERO);
        assert_eq!(r.abstain, Weight::ZERO);
        assert!(r.trail.is_empty());
    }

    #[test]
    fn no_one_voted() {
        let p = ProposalId::new();
        let t = TopicId::new();
        let users = (0..3).map(|_| UserId::new()).collect::<Vec<_>>();
        let r = tally(p, t, &[], &[], &eligible_unit(&users));
        assert_eq!(r.counted_weight(), Weight::ZERO);
        assert_eq!(r.uncounted_weight(), Weight::from(3u32));
        for entry in &r.trail {
            assert!(matches!(
                entry.kind,
                TrailKind::NotCounted {
                    reason: NotCountedReason::NoDirectVoteInChain
                }
            ));
        }
    }

    #[test]
    fn single_direct_yes() {
        let p = ProposalId::new();
        let t = TopicId::new();
        let alice = UserId::new();
        let r = tally(
            p,
            t,
            &[vote(p, alice, VoteChoice::Yes)],
            &[],
            &eligible_unit(&[alice]),
        );
        assert_eq!(r.yes, Weight::ONE);
        assert_eq!(r.no, Weight::ZERO);
        assert_eq!(r.abstain, Weight::ZERO);
        assert!(matches!(
            r.trail[0].kind,
            TrailKind::Direct {
                choice: VoteChoice::Yes
            }
        ));
    }

    #[test]
    fn direct_yes_no_abstain_sum_correctly() {
        let p = ProposalId::new();
        let t = TopicId::new();
        let users = (0..3).map(|_| UserId::new()).collect::<Vec<_>>();
        let r = tally(
            p,
            t,
            &[
                vote(p, users[0], VoteChoice::Yes),
                vote(p, users[1], VoteChoice::No),
                vote(p, users[2], VoteChoice::Abstain),
            ],
            &[],
            &eligible_unit(&users),
        );
        assert_eq!(r.yes, Weight::ONE);
        assert_eq!(r.no, Weight::ONE);
        assert_eq!(r.abstain, Weight::ONE);
    }

    #[test]
    fn one_hop_delegation_carries_weight() {
        // alice → bob; bob votes yes. Both should contribute to yes.
        let p = ProposalId::new();
        let t = TopicId::new();
        let alice = UserId::new();
        let bob = UserId::new();
        let r = tally(
            p,
            t,
            &[vote(p, bob, VoteChoice::Yes)],
            &[deleg(alice, bob, t)],
            &eligible_unit(&[alice, bob]),
        );
        assert_eq!(r.yes, Weight::from(2u32));
        assert_eq!(r.no, Weight::ZERO);
        assert_eq!(r.abstain, Weight::ZERO);

        // alice's trail entry: delegated to bob (terminal), no intermediate hops.
        let alice_entry = r.trail.iter().find(|e| e.user_id == alice).unwrap();
        match &alice_entry.kind {
            TrailKind::Delegated {
                path,
                terminal,
                choice,
            } => {
                assert_eq!(path.len(), 0, "no intermediate hops in 1-hop chain");
                assert_eq!(*terminal, bob);
                assert_eq!(*choice, VoteChoice::Yes);
            }
            other => panic!("expected delegated, got {other:?}"),
        }
    }

    #[test]
    fn two_hop_chain_carries_weight() {
        // alice → bob → carol; carol votes yes. All three contribute.
        let p = ProposalId::new();
        let t = TopicId::new();
        let alice = UserId::new();
        let bob = UserId::new();
        let carol = UserId::new();

        let r = tally(
            p,
            t,
            &[vote(p, carol, VoteChoice::Yes)],
            &[deleg(alice, bob, t), deleg(bob, carol, t)],
            &eligible_unit(&[alice, bob, carol]),
        );
        assert_eq!(r.yes, Weight::from(3u32));

        let alice_entry = r.trail.iter().find(|e| e.user_id == alice).unwrap();
        match &alice_entry.kind {
            TrailKind::Delegated {
                path,
                terminal,
                choice,
            } => {
                assert_eq!(path, &vec![bob], "intermediate hop is bob");
                assert_eq!(*terminal, carol);
                assert_eq!(*choice, VoteChoice::Yes);
            }
            other => panic!("alice should be delegated, got {other:?}"),
        }

        let bob_entry = r.trail.iter().find(|e| e.user_id == bob).unwrap();
        match &bob_entry.kind {
            TrailKind::Delegated {
                path,
                terminal,
                choice,
            } => {
                assert_eq!(path.len(), 0, "bob has no intermediate hop");
                assert_eq!(*terminal, carol);
                assert_eq!(*choice, VoteChoice::Yes);
            }
            other => panic!("bob should be delegated, got {other:?}"),
        }
    }

    #[test]
    fn direct_vote_overrides_delegation() {
        // alice → bob (delegation active); alice ALSO votes directly No.
        // bob votes Yes. alice's direct vote dominates: alice → No, bob → Yes.
        let p = ProposalId::new();
        let t = TopicId::new();
        let alice = UserId::new();
        let bob = UserId::new();

        let r = tally(
            p,
            t,
            &[
                vote(p, alice, VoteChoice::No),
                vote(p, bob, VoteChoice::Yes),
            ],
            &[deleg(alice, bob, t)],
            &eligible_unit(&[alice, bob]),
        );
        assert_eq!(r.yes, Weight::ONE);
        assert_eq!(r.no, Weight::ONE);

        let alice_entry = r.trail.iter().find(|e| e.user_id == alice).unwrap();
        assert!(
            matches!(
                alice_entry.kind,
                TrailKind::Direct {
                    choice: VoteChoice::No
                }
            ),
            "alice's direct vote must dominate delegation"
        );
    }

    #[test]
    fn delegation_on_other_topic_is_ignored() {
        // alice → bob on topic X. The proposal is in topic Y. bob votes yes.
        // alice should contribute NOTHING (no delegation on Y).
        let p = ProposalId::new();
        let topic_x = TopicId::new();
        let topic_y = TopicId::new();
        let alice = UserId::new();
        let bob = UserId::new();

        let r = tally(
            p,
            topic_y,
            &[vote(p, bob, VoteChoice::Yes)],
            &[deleg(alice, bob, topic_x)],
            &eligible_unit(&[alice, bob]),
        );
        assert_eq!(r.yes, Weight::ONE, "only bob's direct vote counts");
        let alice_entry = r.trail.iter().find(|e| e.user_id == alice).unwrap();
        assert!(matches!(alice_entry.kind, TrailKind::NotCounted { .. }));
    }

    #[test]
    fn revoked_delegation_is_ignored() {
        let p = ProposalId::new();
        let t = TopicId::new();
        let alice = UserId::new();
        let bob = UserId::new();

        let r = tally(
            p,
            t,
            &[vote(p, bob, VoteChoice::Yes)],
            &[revoked_deleg(alice, bob, t)],
            &eligible_unit(&[alice, bob]),
        );
        assert_eq!(r.yes, Weight::ONE);
        assert!(matches!(
            r.trail.iter().find(|e| e.user_id == alice).unwrap().kind,
            TrailKind::NotCounted { .. }
        ));
    }

    #[test]
    fn chain_dead_ends_no_contribution() {
        // alice → bob; bob has no further delegation and does not vote.
        // alice contributes nothing; bob contributes nothing.
        let p = ProposalId::new();
        let t = TopicId::new();
        let alice = UserId::new();
        let bob = UserId::new();

        let r = tally(
            p,
            t,
            &[],
            &[deleg(alice, bob, t)],
            &eligible_unit(&[alice, bob]),
        );
        assert_eq!(r.counted_weight(), Weight::ZERO);
        assert_eq!(r.uncounted_weight(), Weight::from(2u32));
    }

    #[test]
    fn ineligible_voter_does_not_contribute() {
        // bob is not in eligible_users; his vote is ignored.
        let p = ProposalId::new();
        let t = TopicId::new();
        let alice = UserId::new();
        let bob = UserId::new();

        let r = tally(
            p,
            t,
            &[
                vote(p, alice, VoteChoice::Yes),
                vote(p, bob, VoteChoice::No),
            ],
            &[],
            &eligible_unit(&[alice]),
        );
        assert_eq!(r.yes, Weight::ONE);
        assert_eq!(r.no, Weight::ZERO);
    }

    #[test]
    fn votes_for_other_proposals_are_ignored() {
        let p = ProposalId::new();
        let other_p = ProposalId::new();
        let t = TopicId::new();
        let alice = UserId::new();

        let r = tally(
            p,
            t,
            &[
                vote(p, alice, VoteChoice::Yes),
                vote(other_p, alice, VoteChoice::No), // unrelated proposal
            ],
            &[],
            &eligible_unit(&[alice]),
        );
        assert_eq!(r.yes, Weight::ONE);
        assert_eq!(r.no, Weight::ZERO);
    }

    #[test]
    fn cyclic_dataset_does_not_loop_and_contributes_nothing() {
        // Defense in depth: cycle detection at write time *should* prevent
        // this, but if A → B → A makes it into the dataset, the tally must
        // not infinite-loop. With neither A nor B voting, both end up
        // NotCounted.
        let p = ProposalId::new();
        let t = TopicId::new();
        let a = UserId::new();
        let b = UserId::new();

        let r = tally(
            p,
            t,
            &[],
            &[deleg(a, b, t), deleg(b, a, t)],
            &eligible_unit(&[a, b]),
        );
        assert_eq!(r.counted_weight(), Weight::ZERO);
        for entry in &r.trail {
            match entry.kind {
                TrailKind::NotCounted {
                    reason: NotCountedReason::DepthExceeded,
                } => { /* expected */ }
                _ => panic!(
                    "cyclic chain should produce DepthExceeded, got {:?}",
                    entry.kind
                ),
            }
        }
    }

    #[test]
    fn cycle_with_one_external_voter_into_it() {
        // Construct: a → b → a (cycle), and c → a (delegating into the cycle).
        // c's chain hits the cycle and never finds a direct vote → DepthExceeded.
        // Nobody votes. All three uncounted.
        let p = ProposalId::new();
        let t = TopicId::new();
        let a = UserId::new();
        let b = UserId::new();
        let c = UserId::new();

        let r = tally(
            p,
            t,
            &[],
            &[deleg(a, b, t), deleg(b, a, t), deleg(c, a, t)],
            &eligible_unit(&[a, b, c]),
        );
        assert_eq!(r.counted_weight(), Weight::ZERO);
    }

    #[test]
    fn abstain_through_delegation() {
        // alice → bob; bob abstains. Both contribute to abstain.
        let p = ProposalId::new();
        let t = TopicId::new();
        let alice = UserId::new();
        let bob = UserId::new();

        let r = tally(
            p,
            t,
            &[vote(p, bob, VoteChoice::Abstain)],
            &[deleg(alice, bob, t)],
            &eligible_unit(&[alice, bob]),
        );
        assert_eq!(r.abstain, Weight::from(2u32));
        assert_eq!(r.yes, Weight::ZERO);
        assert_eq!(r.no, Weight::ZERO);
    }

    #[test]
    fn fan_in_to_one_delegate() {
        // alice, bob, carol all delegate to dave on this topic. dave votes yes.
        // dave's vote carries 4 weight (himself + 3 delegators).
        let p = ProposalId::new();
        let t = TopicId::new();
        let alice = UserId::new();
        let bob = UserId::new();
        let carol = UserId::new();
        let dave = UserId::new();

        let r = tally(
            p,
            t,
            &[vote(p, dave, VoteChoice::Yes)],
            &[
                deleg(alice, dave, t),
                deleg(bob, dave, t),
                deleg(carol, dave, t),
            ],
            &eligible_unit(&[alice, bob, carol, dave]),
        );
        assert_eq!(r.yes, Weight::from(4u32));
    }

    #[test]
    fn deeply_nested_chain_resolves() {
        // Build a chain of 50 users where the last one votes.
        let p = ProposalId::new();
        let t = TopicId::new();
        let chain: Vec<UserId> = (0..50).map(|_| UserId::new()).collect();

        let mut delegations = Vec::new();
        for i in 0..(chain.len() - 1) {
            delegations.push(deleg(chain[i], chain[i + 1], t));
        }
        let votes = vec![vote(p, *chain.last().unwrap(), VoteChoice::Yes)];

        let r = tally(p, t, &votes, &delegations, &eligible_unit(&chain));
        assert_eq!(r.yes, Weight::from(chain.len() as u64));
    }

    #[test]
    fn vote_change_within_window_uses_active_only() {
        // The tally contract: callers pass active votes only (last per (p, v)).
        // We assert the contract by passing a single record and confirming
        // the result reflects it. The "supersession" behavior is the
        // caller's job and lives in civitas-db.
        let p = ProposalId::new();
        let t = TopicId::new();
        let alice = UserId::new();

        // Simulate "alice changed her mind from Yes to No" by passing the
        // active row only.
        let earlier_no = vote(p, alice, VoteChoice::No);
        let active_yes = VoteRecord {
            cast_at: earlier_no.cast_at + Duration::seconds(60),
            ..vote(p, alice, VoteChoice::Yes)
        };

        let r = tally(p, t, &[active_yes], &[], &eligible_unit(&[alice]));
        assert_eq!(r.yes, Weight::ONE);
        assert_eq!(r.no, Weight::ZERO);
    }

    #[test]
    fn tally_is_pure_same_input_same_output() {
        // Determinism: shuffle the input order and confirm the result is
        // identical except possibly for trail ordering, which follows
        // eligible_users ordering.
        let p = ProposalId::new();
        let t = TopicId::new();
        let users: Vec<UserId> = (0..5).map(|_| UserId::new()).collect();
        let votes = vec![
            vote(p, users[0], VoteChoice::Yes),
            vote(p, users[2], VoteChoice::No),
        ];
        let dels = vec![deleg(users[1], users[0], t), deleg(users[3], users[2], t)];
        let elig = eligible_unit(&users);

        let r1 = tally(p, t, &votes, &dels, &elig);

        let mut shuffled_votes = votes.clone();
        shuffled_votes.reverse();
        let mut shuffled_dels = dels.clone();
        shuffled_dels.reverse();
        let r2 = tally(p, t, &shuffled_votes, &shuffled_dels, &elig);

        assert_eq!(r1.yes, r2.yes);
        assert_eq!(r1.no, r2.no);
        assert_eq!(r1.abstain, r2.abstain);
    }
}
