# Architecture

This document is the high-level guide to how Civitas is built and why. Domain deep-dives live under [`docs/architecture/`](./docs/architecture/).

## Layers at a glance

```
┌────────────────────────────────────────────────────────────────┐
│  Browser (SvelteKit SSR + progressive enhancement)            │
└──────────────────────────┬─────────────────────────────────────┘
                           │ HTTPS, JSON over fetch
┌──────────────────────────▼─────────────────────────────────────┐
│  civitas-api  — Axum HTTP server, auth middleware, rate limit │
└──────────────────────────┬─────────────────────────────────────┘
                           │ trait-bound calls (no SQL leaks up)
       ┌───────────────────┴───────────────────┐
       │                                       │
┌──────▼──────┐                          ┌─────▼────────┐
│ civitas-auth│                          │ civitas-core │
│  sessions,  │                          │ pure logic:  │
│  passwords, │                          │ tally, dele- │
│  verify     │                          │ gation, cycle│
└──────┬──────┘                          │ detection    │
       │                                 └─────┬────────┘
       └───────────────┬─────────────────────┬─┘
                       │                     │
                ┌──────▼─────────────────────▼──────┐
                │  civitas-db  — SQLx, migrations   │
                └─────────────────┬─────────────────┘
                                  │
                          ┌───────▼───────┐
                          │ PostgreSQL 16 │
                          └───────────────┘
```

`civitas-types` sits beside everything as a leaf crate of shared domain types.

## Why these technology choices

**Rust + Axum + SQLx.** Boring, fast, type-safe. SQLx's compile-time query checking turns a class of SQL bugs into compile errors. Axum + Tokio is a stable, well-supported web stack. This project will be maintained for years; novelty here is a tax, not a feature.

**SvelteKit.** Server-rendered HTML by default, progressive enhancement, small JS payloads, mobile-friendly. The voting UI must work on slow connections and old devices; SvelteKit's defaults align with that.

**PostgreSQL 16+.** Mature relational store with strong constraints, JSONB where useful, and partial indexes. The vote model leans hard on relational integrity (foreign keys, unique partial indexes for active delegations); a document store would make this harder, not easier.

**TypeScript with strict mode, Tailwind, accessibility primitives.** Clarity over cleverness. The frontend should be readable by someone who hasn't worked on it in six months.

## Crate boundaries (and why they matter)

| Crate          | Role                              | Knows about      |
|----------------|-----------------------------------|------------------|
| `civitas-types`| Shared IDs, enums, value types    | nothing          |
| `civitas-core` | **Pure** voting/delegation logic  | `civitas-types`  |
| `civitas-db`   | SQLx queries, migrations          | core, types      |
| `civitas-auth` | Sessions, passwords, verification | db, types        |
| `civitas-api`  | HTTP, routing, middleware         | all of the above |

The most important boundary is **`civitas-core` knows nothing about a database**. It exposes pure functions that take `&[VoteRecord]` and `&[DelegationRecord]` and return tallies. This means:

1. Voting logic is tested without spinning up Postgres.
2. The tally algorithm can be re-run from a different storage backend later (event-sourced log, blockchain-anchored audit trail) without rewriting the algorithm.
3. Reviewers can audit the logic by reading one crate.

## Core semantic invariants

These hold regardless of how the system evolves:

1. **Append-only votes.** Vote records are never `UPDATE`d. A user changing their vote during an open voting window inserts a new row. Tallies use the most-recent vote per `(proposal_id, voter_id)`.
2. **Direct vote overrides delegation.** If a user voted directly on a proposal, that vote counts; their delegation is not consulted for that proposal.
3. **Delegation is per-topic, transitive, acyclic.** Cycles are rejected at creation time. Chains resolve by walking until a direct voter is found, or the chain ends (no contribution).
4. **Audit trail on every state change.** Every voting-relevant write produces an `audit_log` row.
5. **Soft delete, never hard delete user-visible records.** Recovery and integrity require keeping the trail.

## How the codebase supports the governance migration

The four-phase migration in [GOVERNANCE.md](./GOVERNANCE.md) is not a wish; the codebase has to make it executable.

- **Phase 2** requires the platform itself to support voting on platform changes. The data model already accommodates this: governance proposals are *just* proposals, possibly under a reserved `meta` topic. No new schema needed when the time comes.
- **Phase 3** requires architectural decisions to migrate. Proposals can carry markdown bodies of any size, including RFC-shaped documents. Tooling around proposing and tallying is the same.
- **Phase 4** requires the founder's procedural privileges to be removable. There are none in code: there is no `FOUNDER_ID` env var, no `is_founder` flag, no merge-without-review path. The founder's authority is procedural and social, not encoded; ending Phase 4 is a documentation change and a maintainer-list change, nothing more.

This is deliberate. The codebase must not become a place where founder authority hides.

## Identity and the path to verifiable elections

V1 uses email-and-password with email verification (and optional phone). This is **not** sufficient for binding civic outcomes — we are honest about this in the UI, the docs, and the roadmap.

The architecture is built so that identity providers can be added without rewriting the user model:

- `users` has nullable verification timestamps for email and phone, designed to extend with `eid_verified_at`, `passport_verified_at`, etc.
- The auth crate exposes verification as a trait (`VerificationProvider`) so e-ID, hardware tokens, or KYC integrations slot in alongside the existing flow.
- Voting eligibility checks consult a verification policy, not a hardcoded "email-verified" flag, so the policy can tighten over time.

See [`docs/architecture/identity.md`](./docs/architecture/identity.md) for the full extension plan.

## Path to cryptographic verification

V1 anchors integrity in PostgreSQL: foreign keys, unique constraints, append-only inserts, audit log. This is *integrity by database* and is the right starting point.

V2+ paths under consideration (none committed):

- Per-vote signatures with the user's verification-bound key
- Append-only Merkle log of vote and delegation events, with periodic root publication
- Anchoring of log roots to a public timestamping service or chain

The `civitas-core` purity boundary keeps these options open: the tally algorithm only cares about a stream of records, regardless of how their integrity is established.

See [`docs/architecture/voting.md`](./docs/architecture/voting.md).

## Security model summary

See [SECURITY.md](./SECURITY.md) for vulnerability reporting and guarantees. Key implementation choices:

- Argon2id password hashing
- HTTPS-only `Secure` `HttpOnly` `SameSite=Strict` cookies
- CSRF on all state-changing routes
- Compile-time-checked SQL via SQLx
- Rate limiting on authentication endpoints
- PII redaction in tracing
- Email verification gates voting capability

## What's deliberately out of scope for v1

These belong to later phases and should not be back-ported into v1:

- Blockchain / web3
- KYC integration
- National e-ID integration
- Candidate commitment system (binding representatives to outcomes)
- Real-time push (WebSocket / SSE)
- Native mobile apps
- Internationalization beyond English scaffolding
- Pol.is integration
- Payments
- Public API for third-party clients
- Advanced moderation

Roadmap: [`docs/roadmap.md`](./docs/roadmap.md).
