create extension if not exists citext;

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

create function set_updated_at()
returns trigger
language plpgsql
as $$
begin
    new.updated_at = now();
    return new;
end;
$$;

create table users (
    id uuid primary key,
    email citext not null,
    email_verified_at timestamptz,
    phone text,
    phone_verified_at timestamptz,
    password_hash text not null,
    display_name text not null,
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now(),
    deleted_at timestamptz,
    constraint users_email_uniq unique (email),
    constraint users_phone_uniq unique (phone),
    constraint users_display_name_not_blank check (length(trim(display_name)) > 0)
);

create trigger trg_users_set_updated_at
before update on users
for each row
execute function set_updated_at();

create table topics (
    id uuid primary key,
    slug text not null,
    name text not null,
    description text not null default '',
    created_at timestamptz not null default now(),
    constraint topics_slug_uniq unique (slug),
    constraint topics_slug_format check (slug ~ '^[a-z0-9]+(-[a-z0-9]+)*$'),
    constraint topics_name_not_blank check (length(trim(name)) > 0)
);

create table proposals (
    id uuid primary key,
    topic_id uuid not null,
    title text not null,
    summary text not null,
    body text not null,
    author_id uuid not null,
    status proposal_status not null default 'draft',
    voting_starts_at timestamptz,
    voting_ends_at timestamptz,
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now(),
    constraint proposals_topic_id_fkey
        foreign key (topic_id) references topics (id) on delete restrict,
    constraint proposals_author_id_fkey
        foreign key (author_id) references users (id) on delete restrict,
    constraint proposals_title_not_blank check (length(trim(title)) > 0),
    constraint proposals_summary_not_blank check (length(trim(summary)) > 0),
    constraint proposals_body_not_blank check (length(trim(body)) > 0),
    constraint proposals_voting_window_order check (
        voting_starts_at is null
        or voting_ends_at is null
        or voting_starts_at < voting_ends_at
    )
);

create index idx_proposals_topic_created_at
on proposals (topic_id, created_at desc);

create index idx_proposals_status_voting_window
on proposals (status, voting_starts_at, voting_ends_at);

create trigger trg_proposals_set_updated_at
before update on proposals
for each row
execute function set_updated_at();

create table votes (
    id uuid primary key,
    proposal_id uuid not null,
    voter_id uuid not null,
    choice vote_choice not null,
    weight numeric not null default 1.0,
    cast_at timestamptz not null default now(),
    is_delegated boolean not null default false,
    delegation_chain uuid[],
    constraint votes_proposal_id_fkey
        foreign key (proposal_id) references proposals (id) on delete restrict,
    constraint votes_voter_id_fkey
        foreign key (voter_id) references users (id) on delete restrict,
    constraint votes_weight_non_negative check (weight >= 0)
);

create index idx_votes_proposal_voter_cast_at
on votes (proposal_id, voter_id, cast_at desc);

create index idx_votes_proposal_cast_at
on votes (proposal_id, cast_at desc);

create table delegations (
    id uuid primary key,
    delegator_id uuid not null,
    delegate_id uuid not null,
    topic_id uuid not null,
    created_at timestamptz not null default now(),
    revoked_at timestamptz,
    constraint delegations_delegator_id_fkey
        foreign key (delegator_id) references users (id) on delete restrict,
    constraint delegations_delegate_id_fkey
        foreign key (delegate_id) references users (id) on delete restrict,
    constraint delegations_topic_id_fkey
        foreign key (topic_id) references topics (id) on delete restrict,
    constraint delegations_no_self_delegation check (delegator_id <> delegate_id),
    constraint delegations_revoked_after_created check (
        revoked_at is null or revoked_at >= created_at
    )
);

create unique index delegations_delegator_topic_active_uniq
on delegations (delegator_id, topic_id)
where revoked_at is null;

create index idx_delegations_topic_delegate_active
on delegations (topic_id, delegate_id)
where revoked_at is null;

create table deliberation_comments (
    id uuid primary key,
    proposal_id uuid not null,
    author_id uuid not null,
    parent_id uuid,
    body text not null,
    stance comment_stance not null,
    created_at timestamptz not null default now(),
    constraint deliberation_comments_proposal_id_fkey
        foreign key (proposal_id) references proposals (id) on delete restrict,
    constraint deliberation_comments_author_id_fkey
        foreign key (author_id) references users (id) on delete restrict,
    constraint deliberation_comments_parent_id_fkey
        foreign key (parent_id) references deliberation_comments (id) on delete restrict,
    constraint deliberation_comments_body_not_blank check (length(trim(body)) > 0)
);

create index idx_deliberation_comments_proposal_parent_created_at
on deliberation_comments (proposal_id, parent_id, created_at);

create index idx_deliberation_comments_author_created_at
on deliberation_comments (author_id, created_at desc);

create table audit_log (
    id uuid primary key,
    actor_id uuid,
    action text not null,
    entity_type text not null,
    entity_id uuid not null,
    metadata jsonb not null default '{}'::jsonb,
    created_at timestamptz not null default now(),
    constraint audit_log_actor_id_fkey
        foreign key (actor_id) references users (id) on delete set null,
    constraint audit_log_action_not_blank check (length(trim(action)) > 0),
    constraint audit_log_entity_type_not_blank check (length(trim(entity_type)) > 0)
);

create index idx_audit_log_entity_created_at
on audit_log (entity_type, entity_id, created_at desc);

create index idx_audit_log_actor_created_at
on audit_log (actor_id, created_at desc);
