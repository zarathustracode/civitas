# Voting

Voting is the load-bearing operation of Civitas. This document specifies the algorithm, its invariants, and what makes a tally correct. The implementation lives in `civitas-core`.

## Definitions

- **Direct vote** — a vote cast by a user on a proposal (rows in `votes` with `is_delegated = false`).
- **Active vote** — for a given `(proposal_id, voter_id)`, the row with the latest `cast_at`. All older rows for that pair are superseded.
- **Active delegation** — a row in `delegations` with `revoked_at IS NULL`, scoped to a topic.
- **Voting window** — the open interval `[voting_starts_at, voting_ends_at)` on a proposal in status `voting`. Outside this window, votes are rejected.
- **Tally** — the result of computing `(yes_weight, no_weight, abstain_weight)` for a proposal at a moment in time, including a *trail* describing how each weight flowed.

## Invariants (these never break)

1. A vote cast outside the voting window is rejected at the API layer. The tally function does not need to enforce this; if such a row reaches it, that is a bug, but the algorithm still treats it as data.
2. For any `(proposal_id, voter_id)`, only the active vote contributes to the tally.
3. **Direct vote dominates delegation.** If a user has an active direct vote on a proposal, their delegation for that proposal's topic is not consulted.
4. Delegation is per-topic. A delegation on topic A is irrelevant to a proposal in topic B.
5. Delegation chains are acyclic. Cycles are rejected at delegation *creation* time. The tally algorithm assumes acyclicity; it also defends in depth (max-depth fuse) so a corrupted dataset cannot infinite-loop.
6. Weights compose by addition. A user with weight 1.0 voting carries 1.0; if user A (weight 1.0) delegates to B (weight 1.0) and B votes, B's vote contributes 2.0 to its choice.
7. The tally is a pure function of `(active_votes, active_delegations, proposal_topic_id)`. No clock, no I/O.

## The tally algorithm

Given:

- `votes`: the active vote per `(proposal_id, voter_id)` for the proposal under tally
- `delegations`: the active delegations on the proposal's `topic_id`
- the proposal's `topic_id`

Compute:

```
1. direct_voter_ids ← { v.voter_id | v ∈ votes }
2. for each user U in the system:
     if U ∈ direct_voter_ids:
       contribution(U) = U's direct active vote
     else:
       follow chain: starting at U, repeatedly look up
         delegate_for(current, topic_id) ∈ active_delegations
       until either:
         (a) reach a user X who is in direct_voter_ids → contribute U's weight
             to X's vote
         (b) reach a user X who has no active delegation → no contribution
         (c) exceed MAX_DEPTH → safety abort, no contribution, log anomaly
3. tally:
     yes_weight   = Σ weight(U) where contribution(U).choice = yes
     no_weight    = Σ weight(U) where contribution(U).choice = no
     abstain_weight = Σ weight(U) where contribution(U).choice = abstain
```

In practice the algorithm walks each non-direct voter's chain at most once and memoizes terminal endpoints (a path-compression flavor). For a delegation graph of size N this is `O(N + E)` where E is the number of active delegations.

### Pseudocode (close to the Rust implementation)

```rust
fn tally(
    proposal_topic: TopicId,
    votes: &[VoteRecord],          // active votes for this proposal
    delegations: &[DelegationRecord], // active delegations on this topic
    eligible_users: &[UserWeight], // (user_id, weight)
) -> Tally {
    use std::collections::HashMap;

    let direct: HashMap<UserId, &VoteRecord> =
        votes.iter().map(|v| (v.voter_id, v)).collect();

    let delegate_of: HashMap<UserId, UserId> = delegations
        .iter()
        .filter(|d| d.topic_id == proposal_topic)
        .map(|d| (d.delegator_id, d.delegate_id))
        .collect();

    // Resolve each user once; memoize the terminal vote (or None).
    let mut resolved: HashMap<UserId, Option<VoteChoice>> = HashMap::new();
    let mut yes = Weight::ZERO;
    let mut no = Weight::ZERO;
    let mut abstain = Weight::ZERO;
    let mut trail = Vec::new();

    for u in eligible_users {
        let outcome = resolve(u.user_id, &direct, &delegate_of, &mut resolved, MAX_DEPTH);
        match outcome {
            Some(VoteChoice::Yes)     => yes     += u.weight,
            Some(VoteChoice::No)      => no      += u.weight,
            Some(VoteChoice::Abstain) => abstain += u.weight,
            None => {}
        }
        trail.push(TrailEntry { user_id: u.user_id, outcome, /* path */ });
    }

    Tally { yes, no, abstain, trail }
}

fn resolve(
    start: UserId,
    direct: &HashMap<UserId, &VoteRecord>,
    delegate_of: &HashMap<UserId, UserId>,
    memo: &mut HashMap<UserId, Option<VoteChoice>>,
    max_depth: usize,
) -> Option<VoteChoice> {
    if let Some(cached) = memo.get(&start) { return *cached; }

    let mut current = start;
    let mut visited_in_walk: Vec<UserId> = Vec::new();
    for _ in 0..max_depth {
        if let Some(v) = direct.get(&current) {
            let choice = Some(v.choice);
            for &u in &visited_in_walk { memo.insert(u, choice); }
            memo.insert(current, choice);
            return choice;
        }
        match delegate_of.get(&current) {
            Some(&next) => {
                visited_in_walk.push(current);
                current = next;
            }
            None => {
                for &u in &visited_in_walk { memo.insert(u, None); }
                memo.insert(current, None);
                return None;
            }
        }
    }
    // Depth fuse — should not happen if cycle detection at write-time worked.
    None
}
```

### Why memoization is safe

Once a user's outcome is determined, every other user whose chain ends at the same point shares that outcome. Memoization turns a worst-case `O(N * max_depth)` walk into roughly `O(N)`.

## The trail

Every tally produces a per-user `TrailEntry`:

```rust
pub struct TrailEntry {
    pub user_id: UserId,
    pub path: Vec<UserId>,        // delegation hops (empty for direct voters)
    pub terminal: Option<UserId>, // the user whose vote was actually counted, if any
    pub choice: Option<VoteChoice>,
}
```

The trail makes the tally auditable: given the trail and the underlying vote/delegation records, anyone can reconstruct exactly how the result was reached. The API exposes the aggregate tally to all viewers and the trail to the proposal author and to the user themselves (a user can always see how their own weight flowed).

Whether to expose the full trail publicly is a deployment policy decision. The default is "aggregate only" because chain visibility can leak voting behavior of users who delegated rather than voted directly.

## Vote changes during the voting window

A user with an active vote can cast another vote during the voting window. The new row is inserted; the old row is retained in `votes` for audit. Tallies use the most-recent row per `(proposal_id, voter_id)`.

This means a user's vote can flip before the window closes. Once `voting_ends_at` passes, no further inserts are accepted; the proposal transitions to `closed`; the tally at the moment of close is the final result.

## Concurrency

Within a single Postgres transaction, an `INSERT INTO votes` is serialized at row-level granularity. Two concurrent vote casts for the same `(proposal_id, voter_id)` produce two rows with distinct `cast_at` timestamps and `id`s; the tally simply uses the latter.

For tallying, the API reads at a snapshot via a `READ COMMITTED` transaction. The result is a tally as-of the read time. Two clients tallying simultaneously may see slightly different numbers if a vote lands between their reads — that is acceptable and accurate ("the tally at moment T was X").

## Defense in depth

Even though delegation cycles are rejected at write time, the tally function carries a `MAX_DEPTH` fuse. If a corrupted dataset somehow contains a cycle, the algorithm aborts the offending walk and contributes no weight, rather than looping forever. Such an event is logged at error level for investigation.

## Testing

The test suite for `civitas-core::tally` covers, at minimum:

- Empty proposal (no votes, no delegations) → all-zero tally.
- Direct votes only — yes / no / abstain.
- One-hop delegation: A → B; B votes; A's weight flows.
- Two-hop chain: A → B → C; C votes.
- Direct vote overrides delegation: A → B but A votes directly; A's vote counts.
- Topic isolation: delegation on topic X does not affect a proposal in topic Y.
- Vote change during voting: two votes for same `(p,v)`, the later one wins.
- Cycle in dataset: the safety fuse triggers; no infinite loop; no contribution from the cyclical members.
- Abstain handling: abstain weight is reported separately and never added to yes/no.
- Property-based tests: random graphs of users, delegations, votes — assert `yes + no + abstain ≤ total_eligible_weight`, idempotency, monotonicity under direct-vote replacement.

Tests live in `backend/crates/civitas-core/src/tally/tests.rs` (or split across submodules).

## Path to cryptographic verifiability

V1 anchors integrity in the database: foreign keys, append-only inserts, audit log. Stronger guarantees are layered on top, not into the tally algorithm:

- Per-vote signatures binding `(proposal_id, voter_id, choice, cast_at)` to a key from the verification flow. Verifiable client-side, given the user's public key.
- Append-only Merkle log of vote events. Periodic root publication makes the vote set public and tamper-evident.
- Anchoring of log roots to a public timestamping service.

Each layer is additive. The pure tally function does not change.
