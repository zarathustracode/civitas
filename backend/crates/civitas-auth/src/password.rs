//! Password hashing and verification (Argon2id).
//!
//! Hashing is CPU-bound and runs on a blocking thread so the async runtime
//! is not stalled. Verification is constant-time at the Argon2 layer.

use argon2::password_hash::{
    rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString,
};
use argon2::Argon2;

use crate::{AuthError, AuthResult};

/// Hash a plaintext password with Argon2id default parameters.
///
/// Runs on a `tokio` blocking thread; fine to call from async handlers.
pub async fn hash(plaintext: String) -> AuthResult<String> {
    tokio::task::spawn_blocking(move || hash_blocking(&plaintext))
        .await
        .map_err(|e| AuthError::Internal(format!("join error: {e}")))?
}

/// Verify a plaintext against a stored hash. Returns `Ok(true)` on match,
/// `Ok(false)` on mismatch, `Err` only for malformed hashes.
pub async fn verify(plaintext: String, hash_str: String) -> AuthResult<bool> {
    tokio::task::spawn_blocking(move || verify_blocking(&plaintext, &hash_str))
        .await
        .map_err(|e| AuthError::Internal(format!("join error: {e}")))?
}

fn hash_blocking(plaintext: &str) -> AuthResult<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon = Argon2::default();
    let hash = argon
        .hash_password(plaintext.as_bytes(), &salt)
        .map_err(|e| AuthError::Internal(format!("argon2 hash failed: {e}")))?;
    Ok(hash.to_string())
}

fn verify_blocking(plaintext: &str, stored_hash: &str) -> AuthResult<bool> {
    let parsed = PasswordHash::new(stored_hash)
        .map_err(|e| AuthError::Internal(format!("argon2 parse failed: {e}")))?;
    match Argon2::default().verify_password(plaintext.as_bytes(), &parsed) {
        Ok(()) => Ok(true),
        Err(argon2::password_hash::Error::Password) => Ok(false),
        Err(e) => Err(AuthError::Internal(format!("argon2 verify failed: {e}"))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn hash_and_verify_round_trip() {
        let pw = "correct horse battery staple".to_string();
        let h = hash(pw.clone()).await.unwrap();
        assert!(verify(pw, h.clone()).await.unwrap());
        assert!(!verify("wrong password".to_string(), h).await.unwrap());
    }

    #[tokio::test]
    async fn malformed_hash_errors() {
        let r = verify("x".to_string(), "not-a-real-hash".to_string()).await;
        assert!(r.is_err());
    }
}
