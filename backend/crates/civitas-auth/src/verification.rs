//! Email-verification flow.
//!
//! The [`VerificationProvider`] trait is the extension point: future methods
//! (phone SMS, government e-ID, hardware tokens) implement it alongside the
//! email default. v1 ships only [`EmailVerificationProvider`].

use chrono::{Duration, Utc};
use sqlx::{PgPool, Postgres, Transaction};

use civitas_db::{
    tokens::{self as db_tokens},
    users::{self, User},
};
use civitas_types::UserId;

use crate::tokens::{self, TokenPair};
use crate::{AuthError, AuthResult};

pub const DEFAULT_VERIFICATION_LIFETIME: Duration = Duration::hours(24);

/// Outcome of issuing a verification token: the row id (for audit), and the
/// plaintext to deliver out-of-band (typically embedded in an email link).
#[derive(Debug, Clone)]
pub struct IssuedVerification {
    pub token_id: uuid::Uuid,
    /// The plaintext token. Embed in the verification link; do not log.
    pub plaintext: String,
}

/// What a successful `complete` returned to the caller.
#[derive(Debug, Clone)]
pub struct VerificationResult {
    pub user: User,
}

/// Pluggable verification mechanism. v1 has one impl; future variants
/// (phone, e-id, webauthn) implement this same shape.
///
/// The async methods desugar to `fn -> impl Future + Send` so the trait can
/// be used across `tokio` task boundaries. Implementations can write them
/// as `async fn`; the compiler matches the return type.
pub trait VerificationProvider {
    /// Stable identifier for telemetry and audit.
    fn method(&self) -> &'static str;

    /// Begin a verification — issue a token, return what the caller must
    /// transmit to the user. The transmission itself (email send, SMS) is
    /// the caller's responsibility.
    fn initiate(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        user_id: UserId,
        contact: &str,
    ) -> impl std::future::Future<Output = AuthResult<IssuedVerification>> + Send;

    /// Complete a verification — consume the token and apply the side
    /// effect (e.g. mark email verified). Idempotent.
    fn complete(
        &self,
        pool: &PgPool,
        token_plaintext: &str,
    ) -> impl std::future::Future<Output = AuthResult<VerificationResult>> + Send;
}

/// v1 default: email verification.
pub struct EmailVerificationProvider {
    pub lifetime: Duration,
}

impl Default for EmailVerificationProvider {
    fn default() -> Self {
        Self {
            lifetime: DEFAULT_VERIFICATION_LIFETIME,
        }
    }
}

impl VerificationProvider for EmailVerificationProvider {
    fn method(&self) -> &'static str {
        "email"
    }

    async fn initiate(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        user_id: UserId,
        contact: &str,
    ) -> AuthResult<IssuedVerification> {
        let TokenPair { plaintext, hash } = tokens::generate();
        let expires_at = Utc::now() + self.lifetime;

        let token_id =
            db_tokens::issue_email_verification(tx, user_id, contact, &hash, expires_at).await?;

        Ok(IssuedVerification {
            token_id,
            plaintext,
        })
    }

    async fn complete(
        &self,
        pool: &PgPool,
        token_plaintext: &str,
    ) -> AuthResult<VerificationResult> {
        let hash = tokens::hash_token(token_plaintext);

        let mut tx = pool.begin().await?;
        let consumed = db_tokens::consume_email_verification(&mut tx, &hash).await?;

        let (user_id, email_at_issue) = consumed.ok_or(AuthError::TokenInvalid)?;

        // Confirm the user's current email still matches the email the token
        // was issued for. If they changed it after the token was issued, the
        // token applies to the old address and is rejected.
        let user = users::find_by_id(&mut *tx, user_id)
            .await?
            .ok_or(AuthError::TokenInvalid)?;
        if !user.email.eq_ignore_ascii_case(&email_at_issue) {
            return Err(AuthError::TokenInvalid);
        }

        users::mark_email_verified(&mut tx, user_id).await?;
        let user = users::find_by_id(&mut *tx, user_id)
            .await?
            .ok_or_else(|| AuthError::Internal("user disappeared".into()))?;

        tx.commit().await?;
        Ok(VerificationResult { user })
    }
}
