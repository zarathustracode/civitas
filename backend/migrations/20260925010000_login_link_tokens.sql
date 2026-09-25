-- ─── login_link_tokens (auth) ────────────────────────────────────────────────
--
-- Passwordless sign-in ("magic links"). Same hash-only, one-shot pattern as
-- password_reset_tokens. Lifetimes are short (minutes) because the token
-- alone opens a session, and issuing a new link retires older unused ones.

create table login_link_tokens (
    id          uuid primary key,
    user_id     uuid not null,
    token_hash  text not null,
    created_at  timestamptz not null default now(),
    expires_at  timestamptz not null,
    consumed_at timestamptz,

    constraint login_link_tokens_user_id_fkey
        foreign key (user_id) references users (id) on delete restrict,

    constraint login_link_tokens_token_hash_uniq unique (token_hash),

    constraint login_link_tokens_expires_after_created
        check (expires_at > created_at),
    constraint login_link_tokens_consumed_after_created
        check (consumed_at is null or consumed_at >= created_at)
);

create index idx_login_link_tokens_user_active
    on login_link_tokens (user_id, expires_at)
    where consumed_at is null;
