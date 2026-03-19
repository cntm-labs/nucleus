use base64::Engine;
use ring::rand::SecureRandom;
use sha2::{Digest, Sha256};

/// Generate a PKCE code verifier (43 chars for 32 random bytes, base64url-encoded).
pub fn generate_verifier() -> String {
    let mut bytes = [0u8; 32];
    ring::rand::SystemRandom::new()
        .fill(&mut bytes)
        .expect("system random failed");
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
}

/// Generate a PKCE S256 challenge from a verifier.
pub fn generate_challenge(verifier: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(verifier.as_bytes());
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(hasher.finalize())
}

/// Verify a PKCE challenge against a verifier (S256).
pub fn verify_challenge(verifier: &str, challenge: &str) -> bool {
    generate_challenge(verifier) == challenge
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verifier_is_correct_length() {
        let verifier = generate_verifier();
        // 32 bytes base64url no padding = 43 chars
        assert_eq!(verifier.len(), 43);
    }

    #[test]
    fn challenge_is_deterministic() {
        let verifier = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
        let c1 = generate_challenge(verifier);
        let c2 = generate_challenge(verifier);
        assert_eq!(c1, c2);
    }

    #[test]
    fn verify_challenge_roundtrip() {
        let verifier = generate_verifier();
        let challenge = generate_challenge(&verifier);
        assert!(verify_challenge(&verifier, &challenge));
    }

    #[test]
    fn verify_rejects_wrong_verifier() {
        let verifier = generate_verifier();
        let challenge = generate_challenge(&verifier);
        let wrong_verifier = generate_verifier();
        assert!(!verify_challenge(&wrong_verifier, &challenge));
    }
}
