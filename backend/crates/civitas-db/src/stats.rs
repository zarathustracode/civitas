//! Deployment-wide counts for the read-only operator dashboard.
//!
//! Aggregates only: nothing here returns email addresses or ballot choices.

use sqlx::PgExecutor;

use civitas_types::ProposalStatus;

use crate::DbResult;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UserCounts {
    /// Every account ever registered, including deleted ones.
    pub total: i64,
    /// Active accounts with a verified email — the eligible electorate.
    pub verified: i64,
    /// Active accounts still awaiting email verification.
    pub unverified: i64,
    pub deleted: i64,
    pub registered_last_7_days: i64,
}

pub async fn user_counts<'c, E: PgExecutor<'c>>(conn: E) -> DbResult<UserCounts> {
    let counts = sqlx::query_as!(
        UserCounts,
        r#"
        select
            count(*) as "total!",
            count(*) filter (where deleted_at is null and email_verified_at is not null)
                as "verified!",
            count(*) filter (where deleted_at is null and email_verified_at is null)
                as "unverified!",
            count(*) filter (where deleted_at is not null) as "deleted!",
            count(*) filter (where created_at > now() - interval '7 days')
                as "registered_last_7_days!"
        from users
        "#,
    )
    .fetch_one(conn)
    .await?;
    Ok(counts)
}

/// Proposal counts per status. Statuses with no proposals are absent.
pub async fn proposal_counts_by_status<'c, E: PgExecutor<'c>>(
    conn: E,
) -> DbResult<Vec<(ProposalStatus, i64)>> {
    let rows = sqlx::query!(
        r#"
        select status as "status: ProposalStatus", count(*) as "count!"
        from proposals
        group by status
        "#,
    )
    .fetch_all(conn)
    .await?;
    Ok(rows.into_iter().map(|r| (r.status, r.count)).collect())
}

pub async fn count_active_delegations<'c, E: PgExecutor<'c>>(conn: E) -> DbResult<i64> {
    let count = sqlx::query_scalar!(
        r#"select count(*) as "count!" from delegations where revoked_at is null"#,
    )
    .fetch_one(conn)
    .await?;
    Ok(count)
}

pub async fn count_active_sessions<'c, E: PgExecutor<'c>>(conn: E) -> DbResult<i64> {
    let count = sqlx::query_scalar!(
        r#"
        select count(*) as "count!"
        from sessions
        where revoked_at is null and expires_at > now()
        "#,
    )
    .fetch_one(conn)
    .await?;
    Ok(count)
}
