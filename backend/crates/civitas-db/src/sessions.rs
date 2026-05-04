//! Session storage.
//!
//! Sessions are looked up by the **hash** of the cookie token, not by the
//! token itself, so a database leak cannot be replayed against the running
//! service. Hashing is the auth crate's responsibility; this module accepts
//! a `token_hash` and stores it.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgExecutor, Postgres, Transaction};
use uuid::Uuid;

use civitas_types::UserId;

use crate::audit::{write_log, Action};
use crate::DbResult;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionRow {
    pub id: Uuid,
    pub user_id: UserId,
    pub created_at: DateTime<Utc>,
    pub last_seen_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub user_agent: Option<String>,
    pub ip_address: Option<String>,
}

#[derive(Debug, Clone)]
pub struct NewSession<'a> {
    pub user_id: UserId,
    pub token_hash: &'a str,
    pub expires_at: DateTime<Utc>,
    pub user_agent: Option<&'a str>,
    pub ip_address: Option<&'a str>,
}

pub async fn create(
    tx: &mut Transaction<'_, Postgres>,
    new: NewSession<'_>,
) -> DbResult<SessionRow> {
    let id = Uuid::now_v7();
    let row = sqlx::query_as!(
        SessionRow,
        r#"
        insert into sessions (id, user_id, token_hash, expires_at, user_agent, ip_address)
        values ($1, $2, $3, $4, $5, $6)
        returning
            id, user_id as "user_id: UserId",
            created_at, last_seen_at, expires_at, revoked_at,
            user_agent, ip_address
        "#,
        id,
        new.user_id.into_inner(),
        new.token_hash,
        new.expires_at,
        new.user_agent,
        new.ip_address,
    )
    .fetch_one(&mut **tx)
    .await?;

    write_log(
        &mut **tx,
        Some(row.user_id),
        Action::SessionCreated,
        "session",
        row.id,
        None,
    )
    .await?;

    Ok(row)
}

/// Look up an active (non-revoked, non-expired) session by token hash.
pub async fn find_active_by_token_hash<'c, E: PgExecutor<'c>>(
    conn: E,
    token_hash: &str,
) -> DbResult<Option<SessionRow>> {
    let row = sqlx::query!(
        r#"
        select
            id, user_id as "user_id: UserId",
            created_at, last_seen_at, expires_at, revoked_at,
            user_agent, ip_address
        from sessions
        where token_hash = $1
          and revoked_at is null
          and expires_at > now()
        "#,
        token_hash,
    )
    .fetch_optional(conn)
    .await?;

    Ok(row.map(|r| SessionRow {
        id: r.id,
        user_id: r.user_id,
        created_at: r.created_at,
        last_seen_at: r.last_seen_at,
        expires_at: r.expires_at,
        revoked_at: r.revoked_at,
        user_agent: r.user_agent,
        ip_address: r.ip_address,
    }))
}

/// Update `last_seen_at`. Cheap; callers debounce so this fires at most
/// every minute or so per session.
pub async fn touch_last_seen<'c, E: PgExecutor<'c>>(conn: E, session_id: Uuid) -> DbResult<()> {
    sqlx::query!(
        r#"update sessions set last_seen_at = now() where id = $1"#,
        session_id,
    )
    .execute(conn)
    .await?;
    Ok(())
}

pub async fn revoke(
    tx: &mut Transaction<'_, Postgres>,
    actor_id: UserId,
    session_id: Uuid,
) -> DbResult<()> {
    sqlx::query!(
        r#"update sessions set revoked_at = now() where id = $1 and revoked_at is null"#,
        session_id,
    )
    .execute(&mut **tx)
    .await?;

    write_log(
        &mut **tx,
        Some(actor_id),
        Action::SessionRevoked,
        "session",
        session_id,
        None,
    )
    .await?;
    Ok(())
}

/// Revoke all sessions for a user. Used on password change and account
/// deletion.
pub async fn revoke_all_for_user<'c, E: PgExecutor<'c>>(conn: E, user_id: UserId) -> DbResult<u64> {
    let r = sqlx::query!(
        r#"update sessions set revoked_at = now()
           where user_id = $1 and revoked_at is null"#,
        user_id.into_inner(),
    )
    .execute(conn)
    .await?;
    Ok(r.rows_affected())
}
