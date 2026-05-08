//! Proposals.
//!
//! Status transitions are forward-only:
//! `Draft → Deliberation → Voting → Closed`. The state machine is enforced
//! here at the database layer. The schema also enforces the consistency of
//! the voting window (see `proposals_voting_requires_window`).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgExecutor, Postgres, Transaction};

use civitas_types::{ProposalId, ProposalStatus, TopicId, UserId};

use crate::audit::{write_log, Action};
use crate::{DbError, DbResult};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::FromRow)]
pub struct Proposal {
    pub id: ProposalId,
    pub topic_id: TopicId,
    pub title: String,
    pub summary: String,
    pub body: String,
    pub author_id: UserId,
    pub status: ProposalStatus,
    pub voting_starts_at: Option<DateTime<Utc>>,
    pub voting_ends_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct NewProposal<'a> {
    pub topic_id: TopicId,
    pub author_id: UserId,
    pub title: &'a str,
    pub summary: &'a str,
    pub body: &'a str,
}

pub async fn create(
    tx: &mut Transaction<'_, Postgres>,
    new: NewProposal<'_>,
) -> DbResult<Proposal> {
    let id = ProposalId::new();

    let proposal = sqlx::query_as!(
        Proposal,
        r#"
        insert into proposals (id, topic_id, title, summary, body, author_id, status)
        values ($1, $2, $3, $4, $5, $6, 'draft')
        returning
            id as "id: ProposalId",
            topic_id as "topic_id: TopicId",
            title, summary, body,
            author_id as "author_id: UserId",
            status as "status: ProposalStatus",
            voting_starts_at, voting_ends_at,
            created_at, updated_at
        "#,
        id.into_inner(),
        new.topic_id.into_inner(),
        new.title,
        new.summary,
        new.body,
        new.author_id.into_inner(),
    )
    .fetch_one(&mut **tx)
    .await?;

    write_log(
        &mut **tx,
        Some(new.author_id),
        Action::ProposalCreated,
        "proposal",
        proposal.id.into_inner(),
        None,
    )
    .await?;

    Ok(proposal)
}

pub async fn find_by_id<'c, E: PgExecutor<'c>>(
    conn: E,
    id: ProposalId,
) -> DbResult<Option<Proposal>> {
    let row = sqlx::query_as!(
        Proposal,
        r#"
        select
            id as "id: ProposalId",
            topic_id as "topic_id: TopicId",
            title, summary, body,
            author_id as "author_id: UserId",
            status as "status: ProposalStatus",
            voting_starts_at, voting_ends_at,
            created_at, updated_at
        from proposals
        where id = $1
        "#,
        id.into_inner(),
    )
    .fetch_optional(conn)
    .await?;
    Ok(row)
}

pub async fn list_by_topic<'c, E: PgExecutor<'c>>(
    conn: E,
    topic_id: TopicId,
    status_filter: Option<ProposalStatus>,
) -> DbResult<Vec<Proposal>> {
    let rows = sqlx::query_as!(
        Proposal,
        r#"
        select
            id as "id: ProposalId",
            topic_id as "topic_id: TopicId",
            title, summary, body,
            author_id as "author_id: UserId",
            status as "status: ProposalStatus",
            voting_starts_at, voting_ends_at,
            created_at, updated_at
        from proposals
        where topic_id = $1
          and ($2::proposal_status is null or status = $2::proposal_status)
        order by created_at desc
        "#,
        topic_id.into_inner(),
        status_filter as Option<ProposalStatus>,
    )
    .fetch_all(conn)
    .await?;
    Ok(rows)
}

pub async fn list_by_status<'c, E: PgExecutor<'c>>(
    conn: E,
    status: ProposalStatus,
) -> DbResult<Vec<Proposal>> {
    let rows = sqlx::query_as!(
        Proposal,
        r#"
        select
            id as "id: ProposalId",
            topic_id as "topic_id: TopicId",
            title, summary, body,
            author_id as "author_id: UserId",
            status as "status: ProposalStatus",
            voting_starts_at, voting_ends_at,
            created_at, updated_at
        from proposals
        where status = $1
        order by created_at desc
        "#,
        status as ProposalStatus,
    )
    .fetch_all(conn)
    .await?;
    Ok(rows)
}

/// Find proposals whose voting window has expired (`voting_ends_at <= now`)
/// but whose status is still `voting`, and transition them to `closed`.
/// Each transition is recorded in the audit log with `actor_id = NULL` to
/// distinguish system-initiated closes from author-initiated ones.
///
/// Returns the ids that were closed (typically empty on most ticks). Runs
/// in a single transaction so a partial failure does not leave a mix of
/// open and closed expired proposals.
pub async fn auto_close_expired(tx: &mut Transaction<'_, Postgres>) -> DbResult<Vec<ProposalId>> {
    let rows = sqlx::query!(
        r#"
        update proposals
        set status = 'closed'
        where status = 'voting' and voting_ends_at is not null and voting_ends_at <= now()
        returning id as "id: ProposalId"
        "#,
    )
    .fetch_all(&mut **tx)
    .await?;

    let ids: Vec<ProposalId> = rows.into_iter().map(|r| r.id).collect();
    let metadata = serde_json::json!({ "from": "voting", "to": "closed", "by": "system" });
    for id in &ids {
        write_log(
            &mut **tx,
            None,
            Action::ProposalStatusChanged,
            "proposal",
            id.into_inner(),
            Some(&metadata),
        )
        .await?;
    }
    Ok(ids)
}

/// Transition a proposal to the next status. Enforces the forward-only state
/// machine. When transitioning to `Voting`, both `voting_starts_at` and
/// `voting_ends_at` must be supplied.
pub async fn transition_status(
    tx: &mut Transaction<'_, Postgres>,
    actor_id: UserId,
    id: ProposalId,
    target: ProposalStatus,
    voting_window: Option<(DateTime<Utc>, DateTime<Utc>)>,
) -> DbResult<()> {
    let current = sqlx::query_scalar!(
        r#"select status as "status: ProposalStatus" from proposals where id = $1"#,
        id.into_inner(),
    )
    .fetch_optional(&mut **tx)
    .await?
    .ok_or(DbError::NotFound)?;

    if !current.can_transition_to(target) {
        return Err(DbError::InvalidStateTransition {
            from: current.as_str(),
            to: target.as_str(),
        });
    }

    if matches!(target, ProposalStatus::Voting) {
        let (starts, ends) = voting_window.ok_or(DbError::VotingWindowRequired)?;
        if starts >= ends {
            return Err(DbError::VotingWindowInvalid);
        }
        sqlx::query!(
            r#"
            update proposals
            set status = $2, voting_starts_at = $3, voting_ends_at = $4
            where id = $1
            "#,
            id.into_inner(),
            target as ProposalStatus,
            starts,
            ends,
        )
        .execute(&mut **tx)
        .await?;
    } else {
        sqlx::query!(
            r#"update proposals set status = $2 where id = $1"#,
            id.into_inner(),
            target as ProposalStatus,
        )
        .execute(&mut **tx)
        .await?;
    }

    let metadata = serde_json::json!({
        "from": current.as_str(),
        "to": target.as_str(),
    });
    write_log(
        &mut **tx,
        Some(actor_id),
        Action::ProposalStatusChanged,
        "proposal",
        id.into_inner(),
        Some(&metadata),
    )
    .await?;

    Ok(())
}
