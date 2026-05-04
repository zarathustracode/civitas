//! Password reset flow.
//!
//! `request` issues a token. `complete` validates the token, hashes the new
//! password, updates the user, revokes all existing sessions for that user
//! (so a stolen reset cannot keep the attacker logged in elsewhere), and
//! invalidates outstanding reset tokens.

use chrono::{Duration, Utc};
use sqlx::PgPool;

use civitas_db::{tokens as db_tokens, users};

use crate::tokens::{self as auth_tokens, TokenPair};
use crate::{password, AuthError, AuthResult};

pub const DEFAULT_RESET_LIFETIME: Duration = Duration::hours(1);

#[derive(Debug, Clone)]
pub struct IssuedReset {
    pub token_id: uuid::Uuid,
    /// Plaintext to embed in the reset link. Do not log.
    pub plaintext: String,
}

/// Issue a reset token for a registered user. Returns `Ok(None)` when the
/// email does not match an active account — callers should not be able to
/// enumerate accounts by reset response.
pub async fn request(
    pool: &PgPool,
    email: &str,
    lifetime: Duration,
) -> AuthResult<Option<IssuedReset>> {
    let Some(user) = users::find_by_email(pool, email).await? else {
        return Ok(None);
    };
    if !user.is_active() {
        return Ok(None);
    }

    let TokenPair { plaintext, hash } = auth_tokens::generate();
    let expires_at = Utc::now() + lifetime;

    let mut tx = pool.begin().await?;
    let token_id = db_tokens::issue_password_reset(&mut tx, user.id, &hash, expires_at).await?;
    tx.commit().await?;

    Ok(Some(IssuedReset {
        token_id,
        plaintext,
    }))
}

/// Complete the reset. Returns the `user_id` whose password was changed.
pub async fn complete(pool: &PgPool, token_plaintext: &str, new_password: &str) -> AuthResult<()> {
    if new_password.len() < 12 {
        return Err(AuthError::PasswordTooShort);
    }

    let new_hash = password::hash(new_password.to_string()).await?;
    let token_hash = auth_tokens::hash_token(token_plaintext);

    let mut tx = pool.begin().await?;
    let user_id = db_tokens::consume_password_reset(&mut tx, &token_hash)
        .await?
        .ok_or(AuthError::TokenInvalid)?;

    users::update_password_hash(&mut tx, user_id, &new_hash).await?;
    db_tokens::revoke_all_password_resets(&mut *tx, user_id).await?;

    tx.commit().await?;

    // Revoke all sessions outside the transaction — non-critical for the
    // reset's atomicity.
    civitas_db::sessions::revoke_all_for_user(pool, user_id).await?;

    Ok(())
}
