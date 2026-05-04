//! Civitas authentication and verification.
//!
//! v1 implements email + password (Argon2id) with email verification, plus
//! optional phone verification. The crate is built so that future verification
//! providers (e-ID, hardware tokens, `WebAuthn`) plug in alongside the existing
//! flow without rewriting it.
//!
//! Module map:
//! - [`password`] — Argon2id hash + verify (async, on a blocking thread).
//! - [`tokens`] — random opaque tokens, hashed with SHA-256 for storage.
//! - [`session`] — issue, validate, and revoke session cookies.
//! - [`verification`] — `VerificationProvider` trait + email impl.
//! - [`register`] — registration flow (user create + verification token).
//! - [`login`] — credential check + session issuance.
//! - [`password_reset`] — request + complete.

#![doc(html_root_url = "https://docs.rs/civitas-auth/0.1.0")]

pub mod login;
pub mod password;
pub mod password_reset;
pub mod register;
pub mod session;
pub mod tokens;
pub mod verification;

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("invalid credentials")]
    InvalidCredentials,
    #[error("account not verified")]
    NotVerified,
    #[error("token expired")]
    TokenExpired,
    #[error("token invalid")]
    TokenInvalid,
    #[error("rate limited")]
    RateLimited,
    #[error("invalid email")]
    InvalidEmail,
    #[error("password too short (minimum 12 characters)")]
    PasswordTooShort,
    #[error("display name is required")]
    DisplayNameRequired,
    #[error("email already taken")]
    EmailAlreadyTaken,
    #[error("database: {0}")]
    Db(#[from] civitas_db::DbError),
    #[error("sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("internal error: {0}")]
    Internal(String),
}

pub type AuthResult<T> = std::result::Result<T, AuthError>;
