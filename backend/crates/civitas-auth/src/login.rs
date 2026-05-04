//! Login flow.
//!
//! Looks up the user by email, verifies the password, issues a session.
//! Authentication failures return [`AuthError::InvalidCredentials`] without
//! distinguishing "no such user" from "bad password" — the caller should
//! not be able to enumerate users by timing or response.

use chrono::Duration;
use sqlx::PgPool;

use civitas_db::users;

use crate::session::{self, IssuedSession};
use crate::{password, AuthError, AuthResult};

pub async fn authenticate(
    pool: &PgPool,
    email: &str,
    plaintext_password: &str,
    user_agent: Option<&str>,
    ip_address: Option<&str>,
    session_lifetime: Duration,
) -> AuthResult<IssuedSession> {
    // Always perform the password verification, even on user-not-found, so
    // the timing of "no such user" matches the timing of "bad password."
    // We use a known-bad hash for the dummy verify so the path is realistic.
    const DUMMY_HASH: &str = "$argon2id$v=19$m=19456,t=2,p=1$YWFhYWFhYWFhYWFhYWFhYQ$\
        2fH1cz/CGhwTQA8e0pY2kYJq5b9KXQfQ8j1/9oQ7Fmc";

    let lookup = users::find_password_hash_by_email(pool, email).await?;
    let Some((user_id, stored_hash)) = lookup else {
        // Verify against a dummy to normalize timing, then fail.
        let _ = password::verify(plaintext_password.to_string(), DUMMY_HASH.to_string()).await;
        return Err(AuthError::InvalidCredentials);
    };

    let ok = password::verify(plaintext_password.to_string(), stored_hash).await?;
    if !ok {
        return Err(AuthError::InvalidCredentials);
    }

    let user = users::find_by_id(pool, user_id)
        .await?
        .ok_or(AuthError::InvalidCredentials)?;
    if !user.is_email_verified() {
        return Err(AuthError::NotVerified);
    }

    let mut tx = pool.begin().await?;
    let issued = session::issue(&mut tx, user_id, user_agent, ip_address, session_lifetime).await?;
    tx.commit().await?;
    Ok(issued)
}
