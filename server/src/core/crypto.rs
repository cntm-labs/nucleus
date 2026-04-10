use crate::core::error::AppError;
use aes_gcm::aead::Aead;
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use argon2::password_hash::SaltString;
use argon2::{Algorithm, Argon2, Params, PasswordHash, PasswordHasher, PasswordVerifier, Version};
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use hmac::{Hmac, Mac};
use rand::rngs::OsRng;
use ring::rand::{SecureRandom, SystemRandom};
use sha2::{Digest, Sha256};

// ---------------------------------------------------------------------------
// Password Hashing (Argon2id)
// ---------------------------------------------------------------------------

/// Hash a password using Argon2id (19MB memory, 2 iterations, parallelism 1)
pub fn hash_password(password: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);
    let params = Params::new(19456, 2, 1, None)
        .map_err(|e| anyhow::anyhow!("argon2 params error: {}", e))?;
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| anyhow::anyhow!("password hash error: {}", e))?;
    Ok(hash.to_string())
}

/// Verify a password against an Argon2id hash
pub fn verify_password(password: &str, hash: &str) -> Result<bool, AppError> {
    let parsed_hash =
        PasswordHash::new(hash).map_err(|e| anyhow::anyhow!("password hash parse error: {}", e))?;
    let argon2 = Argon2::default();
    Ok(argon2
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

// ---------------------------------------------------------------------------
// Encryption at Rest (AES-256-GCM)
// ---------------------------------------------------------------------------

/// Generate a 256-bit encryption key
pub fn generate_encryption_key() -> [u8; 32] {
    let rng = SystemRandom::new();
    let mut key = [0u8; 32];
    rng.fill(&mut key).expect("system random failed");
    key
}

/// Encrypt plaintext with AES-256-GCM. Returns nonce || ciphertext.
pub fn encrypt(plaintext: &[u8], key: &[u8; 32]) -> Result<Vec<u8>, AppError> {
    let cipher =
        Aes256Gcm::new_from_slice(key).map_err(|e| anyhow::anyhow!("aes key error: {}", e))?;

    let rng = SystemRandom::new();
    let mut nonce_bytes = [0u8; 12];
    rng.fill(&mut nonce_bytes)
        .map_err(|_| anyhow::anyhow!("random nonce generation failed"))?;
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| anyhow::anyhow!("encryption error: {}", e))?;

    let mut result = Vec::with_capacity(12 + ciphertext.len());
    result.extend_from_slice(&nonce_bytes);
    result.extend_from_slice(&ciphertext);
    Ok(result)
}

/// Decrypt ciphertext (nonce || ciphertext) with AES-256-GCM.
pub fn decrypt(ciphertext: &[u8], key: &[u8; 32]) -> Result<Vec<u8>, AppError> {
    if ciphertext.len() < 12 {
        return Err(anyhow::anyhow!("ciphertext too short: must be at least 12 bytes").into());
    }

    let (nonce_bytes, encrypted) = ciphertext.split_at(12);
    let cipher =
        Aes256Gcm::new_from_slice(key).map_err(|e| anyhow::anyhow!("aes key error: {}", e))?;
    let nonce = Nonce::from_slice(nonce_bytes);

    cipher
        .decrypt(nonce, encrypted)
        .map_err(|e| anyhow::anyhow!("decryption error: {}", e).into())
}

// ---------------------------------------------------------------------------
// Token Generation
// ---------------------------------------------------------------------------

/// Generate a 256-bit cryptographically random token, base64url encoded
pub fn generate_token() -> String {
    let rng = SystemRandom::new();
    let mut bytes = [0u8; 32];
    rng.fill(&mut bytes).expect("system random failed");
    URL_SAFE_NO_PAD.encode(bytes)
}

/// Generate SHA-256 hash of a token (for storage)
pub fn generate_token_hash(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    hex::encode(hasher.finalize())
}

/// Generate a random N-digit numeric code (for OTP)
pub fn generate_otp_code(digits: u32) -> String {
    let rng = SystemRandom::new();
    let mut bytes = [0u8; 4];
    rng.fill(&mut bytes).expect("system random failed");
    let num = u32::from_be_bytes(bytes);
    let modulus = 10u32.pow(digits);
    format!("{:0>width$}", num % modulus, width = digits as usize)
}

// ---------------------------------------------------------------------------
// HMAC
// ---------------------------------------------------------------------------

type HmacSha256 = Hmac<Sha256>;

/// Sign a payload with HMAC-SHA256, returns hex-encoded signature
pub fn hmac_sign(key: &[u8], payload: &[u8]) -> String {
    let mut mac = <HmacSha256 as Mac>::new_from_slice(key).expect("HMAC accepts any key length");
    mac.update(payload);
    hex::encode(mac.finalize().into_bytes())
}

/// Verify HMAC-SHA256 signature (constant-time)
pub fn hmac_verify(key: &[u8], payload: &[u8], signature: &str) -> bool {
    let sig_bytes = match hex::decode(signature) {
        Ok(b) => b,
        Err(_) => return false,
    };
    let mut mac = <HmacSha256 as Mac>::new_from_slice(key).expect("HMAC accepts any key length");
    mac.update(payload);
    mac.verify_slice(&sig_bytes).is_ok()
}

// ---------------------------------------------------------------------------
// Constant-time Comparison
// ---------------------------------------------------------------------------

/// Compare two byte slices in constant time
pub fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut result: u8 = 0;
    for (x, y) in a.iter().zip(b.iter()) {
        result |= x ^ y;
    }
    result == 0
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn password_hash_roundtrip() {
        let password = "SuperSecure123!";
        let hash = hash_password(password).unwrap();
        assert!(verify_password(password, &hash).unwrap());
    }

    #[test]
    fn password_hash_wrong_password_fails() {
        let hash = hash_password("correct_password").unwrap();
        assert!(!verify_password("wrong_password", &hash).unwrap());
    }

    #[test]
    fn password_hash_unique_salt() {
        let password = "SamePassword123!";
        let hash1 = hash_password(password).unwrap();
        let hash2 = hash_password(password).unwrap();
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn aes_gcm_encrypt_decrypt_roundtrip() {
        let key = generate_encryption_key();
        let plaintext = b"hello world";
        let ciphertext = encrypt(plaintext, &key).unwrap();
        let decrypted = decrypt(&ciphertext, &key).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn aes_gcm_rejects_tampered_ciphertext() {
        let key = generate_encryption_key();
        let plaintext = b"hello world";
        let mut ciphertext = encrypt(plaintext, &key).unwrap();
        // Flip a byte in the ciphertext portion (after nonce)
        let last = ciphertext.len() - 1;
        ciphertext[last] ^= 0xFF;
        assert!(decrypt(&ciphertext, &key).is_err());
    }

    #[test]
    fn aes_gcm_unique_nonce() {
        let key = generate_encryption_key();
        let plaintext = b"same data";
        let ct1 = encrypt(plaintext, &key).unwrap();
        let ct2 = encrypt(plaintext, &key).unwrap();
        assert_ne!(ct1, ct2);
    }

    #[test]
    fn aes_gcm_rejects_short_ciphertext() {
        let key = generate_encryption_key();
        let short = vec![0u8; 5];
        assert!(decrypt(&short, &key).is_err());
    }

    #[test]
    fn generate_token_is_correct_length() {
        let token = generate_token();
        // 32 bytes base64url no padding = 43 chars
        assert_eq!(token.len(), 43);
    }

    #[test]
    fn generate_token_is_unique() {
        let t1 = generate_token();
        let t2 = generate_token();
        assert_ne!(t1, t2);
    }

    #[test]
    fn token_hash_is_deterministic() {
        let token = generate_token();
        let h1 = generate_token_hash(&token);
        let h2 = generate_token_hash(&token);
        assert_eq!(h1, h2);
    }

    #[test]
    fn token_hash_is_different_for_different_tokens() {
        let t1 = generate_token();
        let t2 = generate_token();
        assert_ne!(generate_token_hash(&t1), generate_token_hash(&t2));
    }

    #[test]
    fn generate_otp_code_correct_digits() {
        let code = generate_otp_code(6);
        assert_eq!(code.len(), 6);
        assert!(code.chars().all(|c| c.is_ascii_digit()));
    }

    #[test]
    fn hmac_sign_verify_roundtrip() {
        let key = b"secret-key";
        let payload = b"important data";
        let sig = hmac_sign(key, payload);
        assert!(hmac_verify(key, payload, &sig));
    }

    #[test]
    fn hmac_verify_rejects_tampered_payload() {
        let key = b"secret-key";
        let sig = hmac_sign(key, b"original");
        assert!(!hmac_verify(key, b"tampered", &sig));
    }

    #[test]
    fn hmac_verify_rejects_wrong_signature() {
        let key = b"secret-key";
        let payload = b"data";
        assert!(!hmac_verify(key, payload, "deadbeef"));
    }

    #[test]
    fn constant_time_eq_works() {
        assert!(constant_time_eq(b"hello", b"hello"));
        assert!(!constant_time_eq(b"hello", b"world"));
        assert!(!constant_time_eq(b"short", b"longer_slice"));
    }
}
