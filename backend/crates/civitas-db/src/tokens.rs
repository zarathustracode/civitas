//! Email-verification and password-reset tokens.
//!
//! Same hash-only storage pattern as sessions: the plaintext is sent to the
//! user; only the hash is stored. Issuing a new token does not invalidate
//! older ones — the auth layer can choose to revoke them when issuing a
//! replacement.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgExecutor, Postgres, Transaction};
use uuid::Uuid;

use civitas_types::UserId;

use crate::DbResult;

// ─── email verification ──────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EmailVerificationToken {
    pub id: Uuid,
    pub user_id: UserId,
    pub email: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub consumed_at: Option<DateTime<Utc>>,
}

pub async fn issue_email_verification(
    tx: &mut Transaction<'_, Postgres>,
    user_id: UserId,
    email: &str,
    token_hash: &str,
    expires_at: DateTime<Utc>,
) -> DbResult<Uuid> {
    let id = Uuid::now_v7();
    sqlx::query!(
        r#"
        insert into email_verification_tokens (id, user_id, email, token_hash, expires_at)
        values ($1, $2, $3, $4, $5)
        "#,
        id,
        user_id.into_inner(),
        email,
        token_hash,
        expires_at,
    )
    .execute(&mut **tx)
    .await?;
    Ok(id)
}

/// Consume the token if it exists, has not been consumed, and is unexpired.
/// Returns `(user_id, email)` for the auth layer to apply.
pub async fn consume_email_verification(
    tx: &mut Transaction<'_, Postgres>,
    token_hash: &str,
) -> DbResult<Option<(UserId, String)>> {
    let row = sqlx::query!(
        r#"
        update email_verification_tokens
        set consumed_at = now()
        where token_hash = $1 and consumed_at is null and expires_at > now()
        returning user_id as "user_id: UserId", email
        "#,
        token_hash,
    )
    .fetch_optional(&mut **tx)
    .await?;
    Ok(row.map(|r| (r.user_id, r.email)))
}

// ─── password reset ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PasswordResetToken {
    pub id: Uuid,
    pub user_id: UserId,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub consumed_at: Option<DateTime<Utc>>,
}

pub async fn issue_password_reset(
    tx: &mut Transaction<'_, Postgres>,
    user_id: UserId,
    token_hash: &str,
    expires_at: DateTime<Utc>,
) -> DbResult<Uuid> {
    let id = Uuid::now_v7();
    sqlx::query!(
        r#"
        insert into password_reset_tokens (id, user_id, token_hash, expires_at)
        values ($1, $2, $3, $4)
        "#,
        id,
        user_id.into_inner(),
        token_hash,
        expires_at,
    )
    .execute(&mut **tx)
    .await?;
    Ok(id)
}

/// Consume a password-reset token. Returns the `user_id` whose password may
/// now be changed.
pub async fn consume_password_reset(
    tx: &mut Transaction<'_, Postgres>,
    token_hash: &str,
) -> DbResult<Option<UserId>> {
    let row = sqlx::query!(
        r#"
        update password_reset_tokens
        set consumed_at = now()
        where token_hash = $1 and consumed_at is null and expires_at > now()
        returning user_id as "user_id: UserId"
        "#,
        token_hash,
    )
    .fetch_optional(&mut **tx)
    .await?;
    Ok(row.map(|r| r.user_id))
}

/// Used after a successful reset to invalidate any other outstanding tokens.
pub async fn revoke_all_password_resets<'c, E: PgExecutor<'c>>(
    conn: E,
    user_id: UserId,
) -> DbResult<u64> {
    let r = sqlx::query!(
        r#"
        update password_reset_tokens
        set consumed_at = now()
        where user_id = $1 and consumed_at is null
        "#,
        user_id.into_inner(),
    )
    .execute(conn)
    .await?;
    Ok(r.rows_affected())
}
