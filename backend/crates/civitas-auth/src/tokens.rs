//! Random opaque tokens for sessions and verification flows.
//!
//! Each token is 32 bytes from the OS CSPRNG, base64url-encoded for
//! transport. The database stores only the SHA-256 hash (hex-encoded), so
//! a leaked dump cannot be replayed against the running service. The
//! plaintext is given to the user once and forgotten.

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use rand::{rngs::OsRng, RngCore};
use sha2::{Digest, Sha256};

/// Plaintext + storage hash for a freshly issued token.
#[derive(Debug, Clone)]
pub struct TokenPair {
    /// Sent to the user via cookie or email link. Never logged.
    pub plaintext: String,
    /// Stored in the database. Stable, hex-encoded.
    pub hash: String,
}

/// Generate a new 32-byte token.
#[must_use]
pub fn generate() -> TokenPair {
    let mut bytes = [0u8; 32];
    OsRng.fill_bytes(&mut bytes);
    let plaintext = URL_SAFE_NO_PAD.encode(bytes);
    let hash = hash_token(&plaintext);
    TokenPair { plaintext, hash }
}

/// Compute the storage hash of a token plaintext (e.g. one read from a
/// cookie). Always hex-encoded SHA-256, deterministic.
#[must_use]
pub fn hash_token(plaintext: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(plaintext.as_bytes());
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_tokens_are_unique() {
        let a = generate();
        let b = generate();
        assert_ne!(a.plaintext, b.plaintext);
        assert_ne!(a.hash, b.hash);
    }

    #[test]
    fn hash_is_deterministic() {
        assert_eq!(hash_token("abc"), hash_token("abc"));
        assert_ne!(hash_token("abc"), hash_token("abd"));
    }

    #[test]
    fn hash_round_trips_with_generate() {
        let pair = generate();
        assert_eq!(hash_token(&pair.plaintext), pair.hash);
    }

    #[test]
    fn hash_format_is_64_hex_chars() {
        let h = hash_token("anything");
        assert_eq!(h.len(), 64);
        assert!(h.chars().all(|c| c.is_ascii_hexdigit()));
    }
}
