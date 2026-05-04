//! Registration flow.
//!
//! Combines password hashing, user creation, and verification-token issuance
//! in a single transactional sequence. Returns the user and the plaintext
//! verification token; the caller is responsible for transmitting it
//! (typically via an email link).

use sqlx::PgPool;

use civitas_db::{users, DbError};
use civitas_types::UserId;
use validator::ValidateEmail;

use crate::password;
use crate::verification::{EmailVerificationProvider, IssuedVerification, VerificationProvider};
use crate::{AuthError, AuthResult};

// Custom mapping turns the unique-violation into a typed AuthError variant.
// All other DbErrors propagate through the #[from] on AuthError::Db.

#[derive(Debug, Clone)]
pub struct NewRegistration<'a> {
    pub email: &'a str,
    pub password: &'a str,
    pub display_name: &'a str,
}

#[derive(Debug, Clone)]
pub struct Registered {
    pub user_id: UserId,
    pub verification: IssuedVerification,
}

pub async fn register(
    pool: &PgPool,
    new: NewRegistration<'_>,
    provider: &EmailVerificationProvider,
) -> AuthResult<Registered> {
    if !new.email.validate_email() {
        return Err(AuthError::InvalidEmail);
    }
    if new.password.len() < 12 {
        return Err(AuthError::PasswordTooShort);
    }
    if new.display_name.trim().is_empty() {
        return Err(AuthError::DisplayNameRequired);
    }

    let password_hash = password::hash(new.password.to_string()).await?;

    let mut tx = pool.begin().await?;

    let user = users::create(
        &mut tx,
        users::NewUser {
            email: new.email,
            password_hash: &password_hash,
            display_name: new.display_name,
        },
    )
    .await
    .map_err(|e| match e {
        DbError::EmailAlreadyTaken => AuthError::EmailAlreadyTaken,
        other => AuthError::Db(other),
    })?;

    let verification = provider.initiate(&mut tx, user.id, &user.email).await?;

    tx.commit().await?;

    Ok(Registered {
        user_id: user.id,
        verification,
    })
}
