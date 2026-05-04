# Delegation

Civitas implements **per-topic transitive delegation** with cycle detection at creation time and direct-vote override at tally time. This document specifies the rules.

## Properties

| Property | Civitas v1 | Why |
|----------|------------|-----|
| Per-topic | Yes | A user can trust different people on different subjects. Blanket delegation conflates expertise. |
| Transitive | Yes | If you trust B, and B trusts C on this topic, your weight reaches C unless you intervene. |
| Acyclic | Enforced | Cycles produce undefined tallying. Rejected at creation time. |
| Direct-vote override | Yes | The user always retains the last word for any specific proposal. |
| Per-proposal override | Not in v1 | Future feature: temporary delegation override for a single proposal without changing the topic-level default. |
| Liquid (revocable any time) | Yes | A delegation can be revoked at any moment. Future tallies use the new state. |
| Public | Yes by default | Delegation graphs are public to verified users; an operator may restrict to anonymized aggregates. |

## Creating a delegation

Endpoint: `POST /delegations` (body: `{ topic_id, delegate_id }`)

Validation:

1. The delegator must be email-verified.
2. The delegate must exist and be active.
3. `delegator_id ≠ delegate_id` (DB-level CHECK).
4. The delegator must not already have an active delegation on this topic. (DB-level partial UNIQUE; revoke first.)
5. **Cycle check:** the new delegation must not introduce a cycle in the per-topic delegation graph. The check walks from the proposed `delegate_id` along its own delegations on this topic. If we reach `delegator_id`, the new edge would close a cycle and the request is rejected with `409 Conflict` and a clear error code (`delegation.cycle`).

The cycle check is implemented in `civitas-core::delegation::would_create_cycle(...)` as a pure function over the existing delegation set plus the proposed new edge. The DB layer calls it inside the same transaction as the INSERT, so concurrent creation cannot race past it.

### Pseudocode

```rust
pub fn would_create_cycle(
    existing: &[DelegationRecord], // active delegations for this topic
    proposed: ProposedDelegation,
) -> bool {
    let mut current = proposed.delegate_id;
    let mut steps = 0;
    while steps < MAX_DEPTH {
        if current == proposed.delegator_id { return true; }
        match next_delegate(existing, current, proposed.topic_id) {
            Some(next) => { current = next; steps += 1; }
            None => return false,
        }
    }
    // Defensive: if we hit the depth fuse, treat as cyclic.
    true
}
```

`MAX_DEPTH` is chosen high enough that real chains never reach it (e.g. 1024) and low enough to never wedge the system on a corrupted dataset.

## Revoking a delegation

Endpoint: `DELETE /delegations/{id}`

The row is **not** deleted. `revoked_at` is set to `now()`. The audit log records `delegation.revoked` with actor and entity. Future tallies do not consult revoked rows; historical tallies (replayed from the log) consult the state as of that moment.

## Replacing a delegation

To change the delegate for a topic, revoke the existing delegation and create a new one. The API may offer an atomic `PUT /delegations/topic/{topic_id}` that does both in one transaction; this is API sugar over the underlying revoke + create operations.

## Direct-vote override

If a user with an active delegation on topic T directly votes on a proposal in topic T, their direct vote counts and the delegation is **not** consulted for that proposal. The delegation remains active for *future* proposals in that topic.

This rule is unconditional. There is no UI option to "always defer to my delegate" — the user can always reclaim their voice on a specific question. Removing this rule would compromise the principle of user sovereignty and is not under consideration.

## Behavior under chain breakage

If a chain `A → B → C → D` exists and the user being followed has no active vote and no further delegation, the chain ends without contribution. No partial credit, no nearest-direct-voter heuristic.

If a chain reaches a user who has been soft-deleted, treat as no active delegation past that point. (The deleted user's outgoing delegations are inactive once they are deleted.)

## Behavior at the topic boundary

A proposal belongs to exactly one topic. Tally lookup uses *that topic*'s delegation map. A user with delegations on multiple topics has them consulted independently for proposals in each topic. This is the entire point of per-topic delegation.

A user with no delegation on the proposal's topic, who does not vote directly, contributes nothing.

## Visibility and privacy

Default: a user's delegations are visible to other verified users. The list "users delegating to X on topic T" is a queryable resource for verified users.

Rationale: delegation is a public political act. The trust someone receives, and from whom, is information voters reasonably want.

Operator override: a deployment may restrict delegation visibility to aggregates (e.g. "X holds Y delegations on topic T") if the deployment context warrants it (e.g. private organizations, jurisdictions with hostile actors). This is a deployment-time configuration, not user-level privacy. Per-user privacy controls are out of scope for v1.

## Edge cases

- **Self-delegation:** rejected by DB-level CHECK constraint.
- **Delegation to a deleted user:** rejected at creation.
- **Delegation to an unverified user:** allowed at creation, but the delegate's chain endpoint rule applies — if they cannot vote (because they are unverified for the proposal's policy), the chain ends with no contribution.
- **Topic deletion:** topics are not deletable in v1. (If they ever become deletable, all delegations on the topic are revoked atomically.)
- **Mass revocation:** there is no "revoke all" endpoint in v1; revocation is per-row. A future bulk operation would simply call the same revoke path in a loop.

## Future extensions (not in v1)

- **Per-proposal delegation override.** Temporarily delegate to person Y just for proposal P, leaving the topic-level default intact. Implemented as a higher-priority delegation row scoped to a proposal.
- **Delegation expiry.** A delegation can carry an `expires_at` so users do not have to remember to revoke trust they no longer extend.
- **Delegation conditions.** "Delegate to X but only if the proposal carries label `economic`." Speculative.
- **Visible delegate accountability.** UI surface showing the proposals on which a delegate has voted, helping users assess whether their trust is well-placed.

None of these require breaking changes to the v1 schema.
