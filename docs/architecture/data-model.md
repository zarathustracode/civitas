# Data model

The full canonical schema lives in `backend/migrations/`. This document explains the intent behind each table and the invariants that the schema enforces. Read the migrations for the exact column definitions.

All primary keys are UUIDs. We use UUIDv7 for sortability — generated rows are roughly time-ordered, which matters for B-tree locality on the audit log and vote tables.

## `users`

| Column              | Type                  | Notes |
|---------------------|-----------------------|-------|
| `id`                | `uuid` PK             | UUIDv7 |
| `email`             | `citext` UNIQUE NOT NULL | case-insensitive, requires `citext` extension |
| `email_verified_at` | `timestamptz` NULL    | NULL until verified |
| `phone`             | `text` UNIQUE NULL    | E.164 format when present |
| `phone_verified_at` | `timestamptz` NULL    | NULL until verified |
| `password_hash`     | `text` NOT NULL       | Argon2id encoded string |
| `display_name`      | `text` NOT NULL       | shown in UI; not unique |
| `created_at`        | `timestamptz` NOT NULL DEFAULT now() | |
| `updated_at`        | `timestamptz` NOT NULL DEFAULT now() | trigger-maintained |
| `deleted_at`        | `timestamptz` NULL    | soft delete |

**Why nullable verification timestamps instead of booleans?** They double as the time of verification (useful for audit) and they extend naturally — adding `eid_verified_at` later is a column addition, not a remodelling.

**Soft delete only.** Vote and delegation records reference users; hard-deleting a user would break the audit trail. Deletion sets `deleted_at`. The user is treated as inactive for new actions but their historical contributions remain accountable.

## `topics`

| Column        | Type        | Notes |
|---------------|-------------|-------|
| `id`          | `uuid` PK   | |
| `slug`        | `text` UNIQUE NOT NULL | URL-safe identifier |
| `name`        | `text` NOT NULL | |
| `description` | `text` NOT NULL DEFAULT '' | |
| `created_at`  | `timestamptz` NOT NULL DEFAULT now() | |

Topics are the unit of delegation. A user may delegate "fiscal policy" votes to one person and "constitutional amendments" to another.

## `proposals`

| Column            | Type                       | Notes |
|-------------------|----------------------------|-------|
| `id`              | `uuid` PK                  | |
| `topic_id`        | `uuid` FK → topics         | |
| `title`           | `text` NOT NULL            | |
| `summary`         | `text` NOT NULL            | short, used in lists |
| `body`            | `text` NOT NULL            | full markdown body |
| `author_id`       | `uuid` FK → users          | |
| `status`          | enum                       | `draft`, `deliberation`, `voting`, `closed` |
| `voting_starts_at`| `timestamptz` NULL         | required when entering `voting` |
| `voting_ends_at`  | `timestamptz` NULL         | required when entering `voting` |
| `created_at`      | `timestamptz` NOT NULL DEFAULT now() | |
| `updated_at`      | `timestamptz` NOT NULL DEFAULT now() | |

The status transition is a state machine: `draft → deliberation → voting → closed`. Backwards transitions are not permitted. Once `closed`, the proposal is read-only forever.

## `votes` (append-only)

| Column           | Type                                | Notes |
|------------------|-------------------------------------|-------|
| `id`             | `uuid` PK                           | |
| `proposal_id`    | `uuid` FK → proposals               | |
| `voter_id`       | `uuid` FK → users                   | |
| `choice`         | enum                                | `yes`, `no`, `abstain` |
| `weight`         | `numeric` NOT NULL DEFAULT 1.0      | |
| `cast_at`        | `timestamptz` NOT NULL DEFAULT now()| |
| `is_delegated`   | `boolean` NOT NULL DEFAULT false    | true if cast on behalf of others |
| `delegation_chain`| `uuid[]` NULL                      | the chain of delegators if `is_delegated` |

**No UPDATE.** The voting API issues `INSERT` only. To change a vote, the user casts a new vote during the voting window; the most-recent row per `(proposal_id, voter_id)` wins. Old rows are retained for audit.

```sql
CREATE INDEX idx_votes_active ON votes (proposal_id, voter_id, cast_at DESC);
```

## `delegations`

| Column        | Type                  | Notes |
|---------------|-----------------------|-------|
| `id`          | `uuid` PK             | |
| `delegator_id`| `uuid` FK → users     | who is delegating |
| `delegate_id` | `uuid` FK → users     | who receives the delegation |
| `topic_id`    | `uuid` FK → topics    | scoped to a topic |
| `created_at`  | `timestamptz` NOT NULL DEFAULT now() | |
| `revoked_at`  | `timestamptz` NULL    | NULL while active |

Constraints:

- `CHECK (delegator_id <> delegate_id)` — no self-delegation
- `UNIQUE (delegator_id, topic_id) WHERE revoked_at IS NULL` — at most one active delegation per (delegator, topic)
- Cycles are detected at creation time in application code (`civitas-core`). The DB cannot reject cycles via constraint alone because they involve transitive paths.

To "change" a delegation, set `revoked_at` on the existing row and INSERT a new one. History is preserved.

## `deliberation_comments`

| Column        | Type                                | Notes |
|---------------|-------------------------------------|-------|
| `id`          | `uuid` PK                           | |
| `proposal_id` | `uuid` FK → proposals               | |
| `author_id`   | `uuid` FK → users                   | |
| `parent_id`   | `uuid` FK → deliberation_comments NULL | thread parent |
| `body`        | `text` NOT NULL                     | markdown |
| `stance`      | enum                                | `support`, `oppose`, `neutral`, `question` |
| `created_at`  | `timestamptz` NOT NULL DEFAULT now()| |

```sql
CREATE INDEX idx_comments_thread ON deliberation_comments (proposal_id, parent_id, created_at);
```

The `stance` field is structural, not decorative. It allows the UI to summarize positions ("12 in support, 4 opposed, 3 questions") and helps deliberation stay legible in long threads.

## `audit_log`

| Column        | Type                              | Notes |
|---------------|-----------------------------------|-------|
| `id`          | `uuid` PK                         | |
| `actor_id`    | `uuid` FK → users NULL            | NULL for system actions |
| `action`      | `text` NOT NULL                   | e.g. `vote.cast`, `delegation.created` |
| `entity_type` | `text` NOT NULL                   | e.g. `proposal`, `vote` |
| `entity_id`   | `uuid` NOT NULL                   | |
| `metadata`    | `jsonb` NOT NULL DEFAULT '{}'     | |
| `created_at`  | `timestamptz` NOT NULL DEFAULT now() | |

```sql
CREATE INDEX idx_audit_entity ON audit_log (entity_type, entity_id, created_at DESC);
CREATE INDEX idx_audit_actor  ON audit_log (actor_id, created_at DESC);
```

Every voting-relevant write produces an `audit_log` row, written in the same transaction as the change. If the audit row cannot be written, the change is rolled back. This is enforced at the `civitas-db` layer, not by triggers (we want the writing code to be explicit about *what* it logs).

## Action codes used in `audit_log.action`

A non-exhaustive starter list — extend as new operations are added:

- `user.registered`
- `user.email_verified`
- `user.phone_verified`
- `user.deleted`
- `topic.created`
- `proposal.created`
- `proposal.status_changed`
- `vote.cast`
- `delegation.created`
- `delegation.revoked`
- `comment.posted`

## Migrations and evolution

Migrations are forward-only. Once a migration is merged to `main` it is immutable; corrections come as new migrations. The migration history is the historical record of how the schema evolved.

Migration numbering is `NNNN_short_slug.sql`. SQLx runs them in lexical order. Each migration must be reversible *in principle* (we can always write a counter-migration), even if the framework does not store down-migrations.

## Future schema extensions

Anticipated, not committed:

- `verifications` — multi-source identity verification (e-ID, hardware tokens). Replaces the binary verification timestamps on `users` with structured records.
- `proposal_versions` — track edits to proposals during deliberation. Currently we just rely on `updated_at`.
- `delegation_overrides` — temporarily delegate to a different person for a single proposal without changing the topic-level default.
- `vote_signatures` — cryptographic signatures over vote records, enabling client-side verifiability.

None of these require breaking changes to the v1 schema.
