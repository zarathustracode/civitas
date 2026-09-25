//! Passwordless sign-in by emailed link ("magic link").
//!
//! `request` issues a short-lived one-shot token for an active, verified
//! account and retires any earlier unused link. `complete` consumes the
//! token and opens a session. Holding the link proves control of the
//! mailbox, which is the same assurance email verification provides.

use chrono::{Duration, Utc};
use sqlx::PgPool;

use civitas_db::{tokens as db_tokens, users};

use crate::session::{self, IssuedSession};
use crate::tokens::{self as auth_tokens, TokenPair};
use crate::{AuthError, AuthResult};

pub const DEFAULT_LINK_LIFETIME: Duration = Duration::minutes(15);

#[derive(Debug, Clone)]
pub struct IssuedLink {
    /// Plaintext to embed in the sign-in link. Do not log.
    pub plaintext: String,
}

/// Issue a sign-in link. Returns `Ok(None)` unless `email` belongs to an
/// active, verified account — callers must respond identically either way
/// so the endpoint cannot enumerate accounts.
pub async fn request(
    pool: &PgPool,
    email: &str,
    lifetime: Duration,
) -> AuthResult<Option<IssuedLink>> {
    let Some(user) = users::find_by_email(pool, email).await? else {
        return Ok(None);
    };
    if !user.is_active() || !user.is_email_verified() {
        return Ok(None);
    }

    let TokenPair { plaintext, hash } = auth_tokens::generate();
    let expires_at = Utc::now() + lifetime;

    let mut tx = pool.begin().await?;
    db_tokens::revoke_all_login_links(&mut *tx, user.id).await?;
    db_tokens::issue_login_link(&mut tx, user.id, &hash, expires_at).await?;
    tx.commit().await?;

    Ok(Some(IssuedLink { plaintext }))
}

/// Consume a sign-in link and open a session. Unknown, expired, reused, and
/// retired links all fail with [`AuthError::TokenInvalid`], as does a link
/// whose account was deleted after it was sent.
pub async fn complete(
    pool: &PgPool,
    token_plaintext: &str,
    user_agent: Option<&str>,
    ip_address: Option<&str>,
    session_lifetime: Duration,
) -> AuthResult<IssuedSession> {
    let token_hash = auth_tokens::hash_token(token_plaintext);

    let mut tx = pool.begin().await?;
    let user_id = db_tokens::consume_login_link(&mut tx, &token_hash)
        .await?
        .ok_or(AuthError::TokenInvalid)?;

    let user = users::find_by_id(&mut *tx, user_id)
        .await?
        .ok_or(AuthError::TokenInvalid)?;
    if !user.is_active() || !user.is_email_verified() {
        return Err(AuthError::TokenInvalid);
    }

    let issued = session::issue(&mut tx, user_id, user_agent, ip_address, session_lifetime).await?;
    tx.commit().await?;
    Ok(issued)
}
