-- Civitas v0.1 initial schema.
--
-- Forward-only migration. Once applied to any environment beyond local
-- development, this file is immutable; corrections come as new migrations.
--
-- Conventions:
--   - Lowercase keywords.
--   - Tables plural, columns singular.
--   - Indexes named idx_<table>_<purpose> (unique indexes: <table>_..._uniq).
--   - Foreign keys named <table>_<col>_fkey, on delete restrict by default.
--   - Soft delete via deleted_at where applicable; hard delete is not used.
--   - All UUIDs generated client-side (uuid v7) for sortability.

-- ─── extensions and helpers ──────────────────────────────────────────────────

create extension if not exists citext;

-- Generic updated_at trigger function. Tables that own an updated_at column
-- attach this trigger so application code never has to set it explicitly.
create or replace function set_updated_at()
returns trigger
language plpgsql
as $$
begin
    new.updated_at = now();
    return new;
end;
$$;

-- ─── enum types ──────────────────────────────────────────────────────────────

create type proposal_status as enum (
    'draft',
    'deliberation',
    'voting',
    'closed'
);

create type vote_choice as enum (
    'yes',
    'no',
    'abstain'
);

create type comment_stance as enum (
    'support',
    'oppose',
    'neutral',
    'question'
);

-- ─── users ───────────────────────────────────────────────────────────────────
--
-- Verification timestamps are nullable; null means "not verified by this
-- method." When verification methods proliferate (phone SMS, e-id, webauthn),
-- the per-method columns will move into a `verifications` table — that is a
-- non-breaking schema migration, not a re-architecture.

create table users (
    id                 uuid primary key,
    email              citext not null,
    email_verified_at  timestamptz,
    phone              text,
    phone_verified_at  timestamptz,
    password_hash      text not null,
    display_name       text not null,
    created_at         timestamptz not null default now(),
    updated_at         timestamptz not null default now(),
    deleted_at         timestamptz,

    constraint users_email_uniq unique (email),
    constraint users_phone_uniq unique (phone),

    constraint users_display_name_not_blank
        check (length(btrim(display_name)) > 0),

    -- Phone numbers must be in e.164 format when present.
    constraint users_phone_format
        check (phone is null or phone ~ '^\+[1-9][0-9]{6,14}$')
);

create trigger trg_users_set_updated_at
before update on users
for each row
execute function set_updated_at();

-- Listing of active (non-deleted) users by recency.
create index idx_users_active_created_at
    on users (created_at desc)
    where deleted_at is null;

-- ─── topics ──────────────────────────────────────────────────────────────────

create table topics (
    id          uuid primary key,
    slug        text not null,
    name        text not null,
    description text not null default '',
    created_at  timestamptz not null default now(),

    constraint topics_slug_uniq unique (slug),

    constraint topics_slug_format
        check (slug ~ '^[a-z0-9]+(-[a-z0-9]+)*$' and length(slug) between 1 and 64),

    constraint topics_name_not_blank
        check (length(btrim(name)) > 0)
);

-- ─── proposals ───────────────────────────────────────────────────────────────

create table proposals (
    id                uuid primary key,
    topic_id          uuid not null,
    title             text not null,
    summary           text not null,
    body              text not null,
    author_id         uuid not null,
    status            proposal_status not null default 'draft',
    voting_starts_at  timestamptz,
    voting_ends_at    timestamptz,
    created_at        timestamptz not null default now(),
    updated_at        timestamptz not null default now(),

    constraint proposals_topic_id_fkey
        foreign key (topic_id) references topics (id) on delete restrict,
    constraint proposals_author_id_fkey
        foreign key (author_id) references users (id) on delete restrict,

    constraint proposals_title_not_blank   check (length(btrim(title))   > 0),
    constraint proposals_summary_not_blank check (length(btrim(summary)) > 0),
    constraint proposals_body_not_blank    check (length(btrim(body))    > 0),

    -- When both window endpoints are present they must be ordered.
    constraint proposals_voting_window_order check (
        voting_starts_at is null
        or voting_ends_at is null
        or voting_starts_at < voting_ends_at
    ),

    -- Status `voting` requires both window endpoints set.
    constraint proposals_voting_requires_window check (
        status <> 'voting'
        or (voting_starts_at is not null and voting_ends_at is not null)
    )
);

create trigger trg_proposals_set_updated_at
before update on proposals
for each row
execute function set_updated_at();

create index idx_proposals_topic_status_created_at
    on proposals (topic_id, status, created_at desc);

create index idx_proposals_status_created_at
    on proposals (status, created_at desc);

create index idx_proposals_status_voting_window
    on proposals (status, voting_starts_at, voting_ends_at);

create index idx_proposals_author_created_at
    on proposals (author_id, created_at desc);

-- ─── votes ───────────────────────────────────────────────────────────────────
--
-- Append-only. Vote changes during the voting window are inserts; the most-
-- recent row per (proposal_id, voter_id) wins at tally time. There is no
-- update on this table — application code must not issue one.

create table votes (
    id                uuid primary key,
    proposal_id       uuid not null,
    voter_id          uuid not null,
    choice            vote_choice not null,
    weight            numeric not null default 1.0,
    cast_at           timestamptz not null default now(),
    is_delegated      boolean not null default false,
    delegation_chain  uuid[],

    constraint votes_proposal_id_fkey
        foreign key (proposal_id) references proposals (id) on delete restrict,
    constraint votes_voter_id_fkey
        foreign key (voter_id) references users (id) on delete restrict,

    constraint votes_weight_non_negative check (weight >= 0)
);

-- Active-vote lookup: greatest cast_at per (proposal, voter).
create index idx_votes_proposal_voter_cast_at
    on votes (proposal_id, voter_id, cast_at desc);

-- Per-proposal audit scan.
create index idx_votes_proposal_cast_at
    on votes (proposal_id, cast_at desc);

-- Per-voter history.
create index idx_votes_voter_cast_at
    on votes (voter_id, cast_at desc);

-- ─── delegations ─────────────────────────────────────────────────────────────
--
-- Per-topic, transitive, revocable. Cycles are rejected at creation time in
-- application code (civitas-core::would_create_cycle). The partial unique
-- index below enforces "at most one active delegation per (delegator, topic)"
-- — the invariant the cycle check relies on, made race-safe by the index.

create table delegations (
    id            uuid primary key,
    delegator_id  uuid not null,
    delegate_id   uuid not null,
    topic_id      uuid not null,
    created_at    timestamptz not null default now(),
    revoked_at    timestamptz,

    constraint delegations_delegator_id_fkey
        foreign key (delegator_id) references users  (id) on delete restrict,
    constraint delegations_delegate_id_fkey
        foreign key (delegate_id)  references users  (id) on delete restrict,
    constraint delegations_topic_id_fkey
        foreign key (topic_id)     references topics (id) on delete restrict,

    constraint delegations_no_self_delegation
        check (delegator_id <> delegate_id),

    constraint delegations_revoked_after_created
        check (revoked_at is null or revoked_at >= created_at)
);

-- At most one active delegation per (delegator, topic).
create unique index delegations_delegator_topic_active_uniq
    on delegations (delegator_id, topic_id)
    where revoked_at is null;

-- "Who is delegating to me on topic T" — for the delegate's accountability page.
create index idx_delegations_delegate_topic_active
    on delegations (delegate_id, topic_id)
    where revoked_at is null;

-- "All active delegations on topic T" — used at tally time.
create index idx_delegations_topic_active
    on delegations (topic_id)
    where revoked_at is null;

-- Per-user delegation history (active and revoked).
create index idx_delegations_delegator_created_at
    on delegations (delegator_id, created_at desc);

-- ─── deliberation_comments ───────────────────────────────────────────────────
--
-- Threaded markdown comments with a structural stance. v1 has minimal
-- moderation: authors can soft-delete their own comments; moderators can
-- hide comments with a published reason. Both operations preserve the row
-- for audit; the body is replaced with a tombstone in application code.

create table deliberation_comments (
    id            uuid primary key,
    proposal_id   uuid not null,
    author_id     uuid not null,
    parent_id     uuid,
    body          text not null,
    stance        comment_stance not null,
    created_at    timestamptz not null default now(),
    edited_at     timestamptz,
    -- Author soft-delete: body replaced with [deleted by author] in app code.
    deleted_at    timestamptz,
    -- Moderator hide: hidden_reason is published to satisfy transparency.
    hidden_at     timestamptz,
    hidden_reason text,

    constraint deliberation_comments_proposal_id_fkey
        foreign key (proposal_id) references proposals (id) on delete restrict,
    constraint deliberation_comments_author_id_fkey
        foreign key (author_id)   references users     (id) on delete restrict,
    constraint deliberation_comments_parent_id_fkey
        foreign key (parent_id)   references deliberation_comments (id) on delete restrict,

    constraint deliberation_comments_body_not_blank
        check (length(btrim(body)) > 0),

    constraint deliberation_comments_no_self_parent
        check (parent_id is null or parent_id <> id),

    constraint deliberation_comments_hidden_consistent
        check (
            (hidden_at is null and hidden_reason is null)
            or (hidden_at is not null and hidden_reason is not null
                and length(btrim(hidden_reason)) > 0)
        )
);

-- Threaded read order: by proposal, then parent (nulls first for top-level),
-- then chronological within the same parent.
create index idx_deliberation_comments_proposal_parent_created_at
    on deliberation_comments (proposal_id, parent_id nulls first, created_at);

create index idx_deliberation_comments_author_created_at
    on deliberation_comments (author_id, created_at desc);

-- ─── audit_log ───────────────────────────────────────────────────────────────
--
-- Every voting-relevant write produces a row here in the same transaction as
-- the change. Application code (civitas-db) is responsible for writing audit
-- rows alongside operations; we don't use db triggers because we want each
-- writing site to be explicit about *what* it logs.

create table audit_log (
    id           uuid primary key,
    -- Nullable for system-initiated actions (e.g. scheduled close of a
    -- voting window, periodic cleanup). On user deletion the actor_id is
    -- nulled rather than blocking the delete.
    actor_id     uuid,
    action       text not null,
    entity_type  text not null,
    entity_id    uuid not null,
    metadata     jsonb not null default '{}'::jsonb,
    created_at   timestamptz not null default now(),

    constraint audit_log_actor_id_fkey
        foreign key (actor_id) references users (id) on delete set null,

    constraint audit_log_action_not_blank
        check (length(btrim(action)) > 0),
    constraint audit_log_entity_type_not_blank
        check (length(btrim(entity_type)) > 0)
);

-- "History of this entity."
create index idx_audit_log_entity_created_at
    on audit_log (entity_type, entity_id, created_at desc);

-- "What did this user do?" Partial because actor_id is nullable.
create index idx_audit_log_actor_created_at
    on audit_log (actor_id, created_at desc)
    where actor_id is not null;

-- "All `vote.cast` events in time order" — for cross-cutting analytics.
create index idx_audit_log_action_created_at
    on audit_log (action, created_at desc);

-- ─── sessions (auth) ─────────────────────────────────────────────────────────
--
-- Created on successful login, consulted on every authenticated request. The
-- cookie sent to the client carries an opaque random token; the database
-- stores only its hash (sha-256, hex-encoded), so a database leak does not
-- allow session hijacking.

create table sessions (
    id           uuid primary key,
    user_id      uuid not null,
    token_hash   text not null,
    created_at   timestamptz not null default now(),
    last_seen_at timestamptz not null default now(),
    expires_at   timestamptz not null,
    revoked_at   timestamptz,
    user_agent   text,
    -- Stored as text so we don't take a dependency on the inet sqlx feature.
    ip_address   text,

    constraint sessions_user_id_fkey
        foreign key (user_id) references users (id) on delete restrict,

    constraint sessions_token_hash_uniq unique (token_hash),

    constraint sessions_expires_after_created
        check (expires_at > created_at),
    constraint sessions_revoked_after_created
        check (revoked_at is null or revoked_at >= created_at)
);

-- Active sessions for a user — used to enumerate / revoke.
create index idx_sessions_user_active
    on sessions (user_id, expires_at)
    where revoked_at is null;

-- Periodic cleanup: "expired sessions to vacuum out."
create index idx_sessions_expires_at
    on sessions (expires_at);

-- ─── email_verification_tokens (auth) ────────────────────────────────────────
--
-- Issued on registration and on email-change requests. The plaintext token
-- is sent to the user's email; only the hash is stored. The `email` column
-- captures *which* address the token was issued for, so a token issued for
-- the old address fails validation naturally if the user later changes their
-- email.

create table email_verification_tokens (
    id          uuid primary key,
    user_id     uuid not null,
    email       citext not null,
    token_hash  text not null,
    created_at  timestamptz not null default now(),
    expires_at  timestamptz not null,
    consumed_at timestamptz,

    constraint email_verification_tokens_user_id_fkey
        foreign key (user_id) references users (id) on delete restrict,

    constraint email_verification_tokens_token_hash_uniq unique (token_hash),

    constraint email_verification_tokens_expires_after_created
        check (expires_at > created_at),
    constraint email_verification_tokens_consumed_after_created
        check (consumed_at is null or consumed_at >= created_at)
);

-- Active tokens for a user.
create index idx_email_verification_tokens_user_active
    on email_verification_tokens (user_id, expires_at)
    where consumed_at is null;

-- ─── password_reset_tokens (auth) ────────────────────────────────────────────
--
-- Issued on password-reset request. Same hash-only storage pattern as
-- sessions and email verification. Consumption is one-shot.

create table password_reset_tokens (
    id          uuid primary key,
    user_id     uuid not null,
    token_hash  text not null,
    created_at  timestamptz not null default now(),
    expires_at  timestamptz not null,
    consumed_at timestamptz,

    constraint password_reset_tokens_user_id_fkey
        foreign key (user_id) references users (id) on delete restrict,

    constraint password_reset_tokens_token_hash_uniq unique (token_hash),

    constraint password_reset_tokens_expires_after_created
        check (expires_at > created_at),
    constraint password_reset_tokens_consumed_after_created
        check (consumed_at is null or consumed_at >= created_at)
);

create index idx_password_reset_tokens_user_active
    on password_reset_tokens (user_id, expires_at)
    where consumed_at is null;
