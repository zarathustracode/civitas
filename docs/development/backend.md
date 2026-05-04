# Backend development

The backend is a Cargo workspace under `backend/` with five crates:

```
backend/
├── Cargo.toml                   # workspace manifest
├── crates/
│   ├── civitas-types/           # shared domain types
│   ├── civitas-core/            # pure voting & delegation logic
│   ├── civitas-db/              # SQLx queries, migrations
│   ├── civitas-auth/            # sessions, passwords, verification
│   └── civitas-api/             # Axum HTTP server (binary)
├── migrations/                  # SQLx migrations (NNNN_slug.sql)
└── .sqlx/                       # offline query metadata (committed)
```

The dependency graph is acyclic and shallow: `api → auth → db → core → types`, plus `db → core → types` and `auth → db → types`. New crates should respect the layering — never reach upward.

## Working with SQLx

SQLx checks queries against the live database at compile time. Two modes:

**Online** (default): the `DATABASE_URL` in `.env` points at a running Postgres with the migrations applied. `cargo build` runs the queries against the DB to validate them.

**Offline:** the `.sqlx/` directory contains JSON-serialized query metadata captured by `cargo sqlx prepare`. CI uses `SQLX_OFFLINE=true` so it does not need a database.

When you add or change a query, regenerate the offline cache:

```bash
cd backend
cargo sqlx prepare --workspace
git add .sqlx/
```

Commit `.sqlx/` along with the code change. CI will fail if a query lacks a cached metadata entry.

### Migrations

Add migrations as files in `backend/migrations/`:

```bash
cd backend
sqlx migrate add -r short_description
```

Forward migrations only — once a migration is merged to `main`, edits are forbidden. To correct an issue, write a new migration. Migration filenames are `NNNN_short_description.sql` (or `.up.sql` / `.down.sql` if reversibility tooling is used).

Apply pending migrations:

```bash
make migrate
# or
cd backend && sqlx migrate run
```

Reset the database (destroys data):

```bash
make db-reset
```

## Crate-by-crate

### `civitas-types`

Shared types only. Newtypes for IDs (`UserId`, `ProposalId`, `TopicId`, `VoteId`, `DelegationId`), enums (`VoteChoice`, `ProposalStatus`, `Stance`), value objects (`Weight`).

Derives `Serialize`, `Deserialize`, `Hash`, `Eq`, etc. Where useful, also derives `ts_rs::TS` to generate TypeScript types for the frontend.

No dependencies on other workspace crates. No I/O.

### `civitas-core`

The most important crate. Contains:

- `tally` — pure tally function over `(votes, delegations, eligible_users)`.
- `delegation::would_create_cycle` — pure cycle detection.
- `eligibility` — pure policy evaluation (does this user satisfy the policy?).

No DB, no HTTP, no clock, no randomness. Deterministic functions only.

If you find yourself reaching for I/O or current time, the responsibility belongs in another crate. Pass values in.

Tests live in `src/**/tests.rs`. Aim for ≥80% line coverage; aim for property tests over random graphs for tally and cycle detection.

### `civitas-db`

All SQLx queries and migrations. Exposes typed functions like `record_vote(user, proposal, choice) -> Result<Vote>` to upstream crates. Internal helpers may use `query!` / `query_as!`.

Audit log writes are colocated with the operation that produces them, in the same transaction:

```rust
pub async fn record_vote(tx: &mut Transaction<'_, Postgres>, …) -> Result<VoteId> {
    let vote_id = sqlx::query_scalar!(...).fetch_one(&mut **tx).await?;
    sqlx::query!(
        "INSERT INTO audit_log (actor_id, action, entity_type, entity_id, metadata)
         VALUES ($1, 'vote.cast', 'vote', $2, $3)",
        actor_id, vote_id, metadata
    ).execute(&mut **tx).await?;
    Ok(vote_id)
}
```

Always pass an existing `Transaction` for multi-step operations; never start a new connection in the middle of a logical operation.

### `civitas-auth`

Sessions, password hashing (Argon2id), email/phone verification, password reset.

The `VerificationProvider` trait abstracts over verification methods. v1 implements `EmailVerificationProvider`. Future providers (`PhoneSmsProvider`, `EidProvider`, `WebAuthnProvider`) implement the same trait.

Session tokens are HTTP-only cookies. Token format and storage strategy is documented in `crates/civitas-auth/README.md`.

### `civitas-api`

Axum routes, middleware, error handling.

Route modules live in `src/routes/`. Each module owns its handler functions and exposes a `router()` that returns an `axum::Router`. The top-level `App::router()` mounts them.

Middleware order (top of `src/lib.rs`):

1. `tower_http::trace::TraceLayer` — request tracing
2. `tower_governor` — rate limiting
3. `tower_http::cors::CorsLayer` — CORS
4. `axum::middleware::from_fn(csrf)` — CSRF
5. `axum::middleware::from_fn(security_headers)` — CSP, HSTS, etc.

Errors implement `IntoResponse` with stable error codes for the client (e.g. `delegation.cycle`, `vote.outside_window`). Never leak internal error messages to the client; log them with `tracing::error!` and return a generic message with the stable code.

## Adding an HTTP endpoint

1. Define request/response types in `civitas-types` (or in the route module if API-shaped only).
2. Add the SQL operation in `civitas-db` if it touches storage.
3. Add the pure logic in `civitas-core` if it computes something testable in isolation.
4. Wire the handler in `civitas-api/src/routes/<module>.rs`.
5. Add an integration test in `civitas-api/tests/`.
6. Update the OpenAPI doc (`docs/api/openapi.yaml`) if present.

## Logging

Use `tracing` with structured fields:

```rust
tracing::info!(user_id = %user.id, proposal_id = %proposal.id, "vote cast");
```

Never log:

- passwords or password hashes
- session tokens
- raw email or phone
- request bodies that may contain PII

Configure log filtering via `RUST_LOG` (see `.env.example`).

## Performance budgets

Read endpoints: P99 < 200 ms locally on warm cache. Write endpoints: P99 < 500 ms.

If a query exceeds these budgets, profile with `EXPLAIN ANALYZE`, add an index if appropriate, and add a benchmark test that fails when the regression returns.
