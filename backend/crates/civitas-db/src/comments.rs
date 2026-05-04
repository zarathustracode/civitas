//! Deliberation comments.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgExecutor, Postgres, Transaction};

use civitas_types::{CommentId, ProposalId, ProposalStatus, Stance, UserId};

use crate::audit::{write_log, Action};
use crate::{DbError, DbResult};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommentRow {
    pub id: CommentId,
    pub proposal_id: ProposalId,
    pub author_id: UserId,
    pub parent_id: Option<CommentId>,
    pub body: String,
    pub stance: Stance,
    pub created_at: DateTime<Utc>,
    pub edited_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub hidden_at: Option<DateTime<Utc>>,
    pub hidden_reason: Option<String>,
}

#[derive(Debug, Clone)]
pub struct NewComment<'a> {
    pub proposal_id: ProposalId,
    pub author_id: UserId,
    pub parent_id: Option<CommentId>,
    pub body: &'a str,
    pub stance: Stance,
}

pub async fn create(
    tx: &mut Transaction<'_, Postgres>,
    new: NewComment<'_>,
) -> DbResult<CommentRow> {
    let status = sqlx::query_scalar!(
        r#"select status as "status: ProposalStatus" from proposals where id = $1"#,
        new.proposal_id.into_inner(),
    )
    .fetch_optional(&mut **tx)
    .await?
    .ok_or(DbError::NotFound)?;

    if !status.accepts_comments() {
        return Err(DbError::CommentsNotAllowedInStatus(status.as_str()));
    }

    if let Some(parent_id) = new.parent_id {
        let parent_proposal = sqlx::query_scalar!(
            r#"select proposal_id as "proposal_id: ProposalId"
               from deliberation_comments where id = $1"#,
            parent_id.into_inner(),
        )
        .fetch_optional(&mut **tx)
        .await?
        .ok_or(DbError::NotFound)?;
        if parent_proposal != new.proposal_id {
            return Err(DbError::CommentParentMismatch);
        }
    }

    let id = CommentId::new();
    let row = sqlx::query_as!(
        CommentRow,
        r#"
        insert into deliberation_comments
            (id, proposal_id, author_id, parent_id, body, stance)
        values ($1, $2, $3, $4, $5, $6)
        returning
            id          as "id: CommentId",
            proposal_id as "proposal_id: ProposalId",
            author_id   as "author_id: UserId",
            parent_id   as "parent_id: CommentId",
            body, stance as "stance: Stance",
            created_at, edited_at, deleted_at, hidden_at, hidden_reason
        "#,
        id.into_inner(),
        new.proposal_id.into_inner(),
        new.author_id.into_inner(),
        new.parent_id.map(CommentId::into_inner),
        new.body,
        new.stance as Stance,
    )
    .fetch_one(&mut **tx)
    .await?;

    write_log(
        &mut **tx,
        Some(new.author_id),
        Action::CommentPosted,
        "comment",
        row.id.into_inner(),
        None,
    )
    .await?;

    Ok(row)
}

pub async fn list_thread<'c, E: PgExecutor<'c>>(
    conn: E,
    proposal_id: ProposalId,
) -> DbResult<Vec<CommentRow>> {
    let rows = sqlx::query!(
        r#"
        select
            id          as "id: CommentId",
            proposal_id as "proposal_id: ProposalId",
            author_id   as "author_id: UserId",
            parent_id   as "parent_id: CommentId",
            body, stance as "stance: Stance",
            created_at, edited_at, deleted_at, hidden_at, hidden_reason
        from deliberation_comments
        where proposal_id = $1
        order by parent_id nulls first, created_at
        "#,
        proposal_id.into_inner(),
    )
    .fetch_all(conn)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| CommentRow {
            id: r.id,
            proposal_id: r.proposal_id,
            author_id: r.author_id,
            parent_id: r.parent_id,
            body: r.body,
            stance: r.stance,
            created_at: r.created_at,
            edited_at: r.edited_at,
            deleted_at: r.deleted_at,
            hidden_at: r.hidden_at,
            hidden_reason: r.hidden_reason,
        })
        .collect())
}

/// Author-initiated soft delete. Idempotent.
pub async fn delete_by_author(
    tx: &mut Transaction<'_, Postgres>,
    actor_id: UserId,
    comment_id: CommentId,
) -> DbResult<()> {
    let updated = sqlx::query!(
        r#"
        update deliberation_comments
        set deleted_at = now()
        where id = $1 and author_id = $2 and deleted_at is null
        "#,
        comment_id.into_inner(),
        actor_id.into_inner(),
    )
    .execute(&mut **tx)
    .await?;

    if updated.rows_affected() == 0 {
        return Err(DbError::NotFound);
    }

    write_log(
        &mut **tx,
        Some(actor_id),
        Action::CommentDeleted,
        "comment",
        comment_id.into_inner(),
        None,
    )
    .await?;
    Ok(())
}

/// Moderator-initiated hide. The reason is published.
pub async fn hide_by_moderator(
    tx: &mut Transaction<'_, Postgres>,
    moderator_id: UserId,
    comment_id: CommentId,
    reason: &str,
) -> DbResult<()> {
    if reason.trim().is_empty() {
        return Err(DbError::ReasonRequired);
    }

    let updated = sqlx::query!(
        r#"
        update deliberation_comments
        set hidden_at = now(), hidden_reason = $2
        where id = $1 and hidden_at is null
        "#,
        comment_id.into_inner(),
        reason,
    )
    .execute(&mut **tx)
    .await?;

    if updated.rows_affected() == 0 {
        return Err(DbError::NotFound);
    }

    let metadata = serde_json::json!({ "reason": reason });
    write_log(
        &mut **tx,
        Some(moderator_id),
        Action::CommentHidden,
        "comment",
        comment_id.into_inner(),
        Some(&metadata),
    )
    .await?;
    Ok(())
}
