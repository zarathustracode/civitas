# Architecture overview

This document expands the summary in [`/ARCHITECTURE.md`](../../ARCHITECTURE.md) with the rationale behind the major design decisions. If you want the one-page version, read the root document. If you want to know *why* a decision was made — and what would have to change for it to be revisited — read this one.

## Design principles, in priority order

When two principles conflict, the higher one wins.

1. **Integrity over performance.** A tally that is fast but wrong is worse than a tally that is slow and correct. Vote records are append-only even when an in-place update would be faster.
2. **Auditability over convenience.** Every state change is logged with actor, action, entity, and timestamp. Yes, this costs storage. The cost is the price of legitimacy.
3. **Boring technology over novelty.** Postgres beats a clever distributed store. Server-rendered HTML beats a SPA-of-the-month. The political work is hard enough; the tech should not also be experimental.
4. **Reversibility for users, irreversibility for history.** Users can change votes during voting windows and revoke delegations any time. The system must never lose the record of what happened.
5. **Subsidiarity in the codebase.** Each crate does one thing. The web layer does not know about SQL. The core does not know about HTTP. This is enforced by crate boundaries, not by convention.

## Layered architecture

Five Rust crates, three layers conceptually:

**Pure layer (no I/O):**
- `civitas-types` — IDs, enums, value objects.
- `civitas-core` — voting and delegation logic as pure functions.

**Adapter layer:**
- `civitas-db` — SQLx queries and migrations.
- `civitas-auth` — sessions, password hashing, verification flows.

**Edge layer:**
- `civitas-api` — HTTP routes, middleware, error responses.

The pure layer is the audit surface. Reviewers should be able to read `civitas-core` end-to-end and convince themselves the algorithm is correct. The adapter layer translates between persistent state and the pure layer. The edge layer is where authentication, authorization, and request shape live.

The frontend (SvelteKit) is a separate codebase under `frontend/`. It talks to `civitas-api` over JSON. The two share types via generated TypeScript declarations from `civitas-types`.

## Data flow: a vote being cast

```
Browser ─POST /proposals/{id}/votes──> civitas-api
                                          │
                                          ├─ session check (civitas-auth)
                                          ├─ rate limit, CSRF, body validation
                                          │
                                          └─> civitas-db::record_vote()
                                                │
                                                ├─ INSERT into votes (append-only)
                                                ├─ INSERT into audit_log
                                                └─ COMMIT
                                          ◄────┘
                                          │
                                          └─ 201 Created (vote receipt)
```

A tally request:

```
Browser ─GET /proposals/{id}/tally──> civitas-api
                                          │
                                          ├─ civitas-db::load_active_votes(proposal_id)
                                          ├─ civitas-db::load_active_delegations(topic_id)
                                          │
                                          └─> civitas-core::tally(&votes, &delegations)
                                                │
                                                └─ returns Tally { yes, no, abstain, trail }
                                          ◄────┘
                                          │
                                          └─ 200 OK (JSON)
```

`civitas-core::tally` is a pure function. Same input, same output, every time. No clock, no I/O, no randomness.

## Why each crate exists

### `civitas-types`
Newtypes for IDs prevent mix-ups (passing a `UserId` where a `ProposalId` is expected fails to compile). Enums for choice and status give exhaustive matching. Putting these in a leaf crate means they can be referenced from anywhere without creating a dependency cycle, and they can be mirrored to TypeScript via `ts-rs`.

### `civitas-core`
This is the most important crate. It contains the answer to "given these votes and these delegations, what is the result?" — and nothing else. By keeping it free of I/O it becomes:
- testable with property-based tests over generated input
- reviewable as a single coherent piece of logic
- replaceable with an alternative storage backend in the future without rewriting the math

### `civitas-db`
Holds all SQLx queries and the migration history. The query macros are compile-time-checked against the schema, which catches a large class of "I forgot to update that query when I added the column" bugs. Migrations are forward-only — once merged to main they are immutable.

### `civitas-auth`
Sessions, password hashing (Argon2id), email/phone verification, account recovery. Designed for extension to e-ID, hardware tokens, and other verification methods through a `VerificationProvider` trait. v1 implements only email + password with email verification.

### `civitas-api`
The HTTP edge. Routes, middleware (rate limiting, CSRF, CSP headers, tracing), error mapping (`anyhow` / domain errors → HTTP status codes with stable error codes for the client). Thin: business logic does not live here.

## Frontend architecture

SvelteKit with TypeScript and Tailwind. Mobile-first. Server-rendered by default; client-side hydration only where genuinely needed.

- `src/routes/` — page routes
- `src/lib/components/` — reusable UI primitives
- `src/lib/api/` — typed fetch wrappers around `civitas-api`
- `src/lib/stores/` — minimal client state (auth status, current user)
- `src/lib/types/` — generated from `civitas-types` via `ts-rs`

The voting interface is the most important UI. It must be unambiguous: every action shows a confirmation step that restates exactly what the user is about to do, and every successful vote shows a receipt with a permalink.

## What's deliberately *not* in the architecture

- **No microservices.** A single Rust binary with a single Postgres database is sufficient and will remain sufficient until measured load says otherwise.
- **No CQRS / event sourcing.** Append-only vote tables and an audit log give us the audit benefits without the complexity. We may add an event log later if integration with external systems makes it useful.
- **No GraphQL.** REST-ish JSON over HTTP. Easier to cache, easier to debug, easier to rate-limit.
- **No SSR-only / no SPA-only.** Progressive enhancement: pages render as HTML, become interactive when JS loads.

When any of these constraints feels limiting, write down *why*. The decision to add complexity is one we want to take in writing, not by drift.

## Evolution path

The boundary between layers is the durable contract. The implementation behind any boundary can be replaced.

- Replacing PostgreSQL with another store: rewrite `civitas-db`. The signatures it exposes to `civitas-api` and the records it returns to `civitas-core` are the contract.
- Replacing email/password with e-ID: extend `civitas-auth`. The user model already has nullable verification timestamps that generalize to other identity sources.
- Adding cryptographic vote anchoring: emit signed events when writing votes. The `civitas-core` algorithm is unchanged.

Each of these is significant work. None of them require rewriting the parts of the system that exist today.
