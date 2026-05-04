//! Session lifecycle.
//!
//! Issuing a session generates an opaque token, stores its hash, and returns
//! the plaintext (to be set as a `Secure HttpOnly SameSite=Strict` cookie).
//! Validation hashes the cookie value and looks up the row.

use chrono::{Duration, Utc};
use sqlx::{PgPool, Postgres, Transaction};

use civitas_db::{
    sessions::{self, SessionRow},
    users::{self, User},
};
use civitas_types::UserId;

use crate::tokens::{self, TokenPair};
use crate::AuthResult;

/// Default session lifetime when the caller has no other policy in mind.
pub const DEFAULT_LIFETIME: Duration = Duration::days(30);

/// A freshly-issued session, returned to the caller so the cookie can be set.
#[derive(Debug, Clone)]
pub struct IssuedSession {
    pub row: SessionRow,
    /// The opaque cookie value. Send once; the server never sees it again
    /// in plaintext.
    pub cookie_value: String,
}

/// Issue a session for `user_id`. Writes the session row and audit log in
/// the same transaction.
pub async fn issue(
    tx: &mut Transaction<'_, Postgres>,
    user_id: UserId,
    user_agent: Option<&str>,
    ip_address: Option<&str>,
    lifetime: Duration,
) -> AuthResult<IssuedSession> {
    let TokenPair { plaintext, hash } = tokens::generate();
    let expires_at = Utc::now() + lifetime;

    let row = sessions::create(
        tx,
        sessions::NewSession {
            user_id,
            token_hash: &hash,
            expires_at,
            user_agent,
            ip_address,
        },
    )
    .await?;

    Ok(IssuedSession {
        row,
        cookie_value: plaintext,
    })
}

/// Look up the active session and user that a cookie belongs to. Touches
/// `last_seen_at` opportunistically.
pub async fn validate(pool: &PgPool, cookie_value: &str) -> AuthResult<Option<(SessionRow, User)>> {
    let hash = tokens::hash_token(cookie_value);

    let Some(session) = sessions::find_active_by_token_hash(pool, &hash).await? else {
        return Ok(None);
    };

    let Some(user) = users::find_by_id(pool, session.user_id).await? else {
        return Ok(None);
    };
    if !user.is_active() {
        return Ok(None);
    }

    // Best-effort touch — ignore errors so a flaky last_seen update does not
    // sink an authenticated request.
    if let Err(e) = sessions::touch_last_seen(pool, session.id).await {
        tracing::debug!(error = ?e, "session touch failed");
    }

    Ok(Some((session, user)))
}

pub async fn revoke(
    tx: &mut Transaction<'_, Postgres>,
    actor_id: UserId,
    session_id: uuid::Uuid,
) -> AuthResult<()> {
    sessions::revoke(tx, actor_id, session_id).await?;
    Ok(())
}

/// Revoke every active session for `user_id` — used on password change and
/// account deletion.
pub async fn revoke_all_for_user(pool: &PgPool, user_id: UserId) -> AuthResult<u64> {
    Ok(sessions::revoke_all_for_user(pool, user_id).await?)
}
