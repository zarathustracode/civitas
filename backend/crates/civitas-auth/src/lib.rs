//! Civitas authentication and verification.
//!
//! v1 implements email + password (Argon2id) with email verification, plus
//! optional phone verification. The crate is built so that future verification
//! providers (e-ID, hardware tokens, `WebAuthn`) plug in alongside the existing
//! flow without rewriting it.
//!
//! Implementation lands in a follow-up session; this is the wiring stub.

#![doc(html_root_url = "https://docs.rs/civitas-auth/0.1.0")]

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
    #[error("internal error: {0}")]
    Internal(String),
}

pub type AuthResult<T> = std::result::Result<T, AuthError>;
