use base64::Engine;
use chrono::Utc;
use nucleus_core::crypto;
use nucleus_core::error::AppError;
use ring::hmac;
use ring::rand::SecureRandom;

pub struct MfaService;

/// TOTP enrollment result
pub struct TotpEnrollment {
    /// Encrypted TOTP secret (store in DB)
    pub secret_enc: String,
    /// URI for QR code (show to user)
    pub totp_uri: String,
    /// The raw secret in base32 (show to user as backup)
    pub secret_base32: String,
}

impl MfaService {
    /// Enroll a new TOTP authenticator.
    /// Generates a random secret, encrypts it, and returns QR URI.
    pub fn enroll_totp(
        user_email: &str,
        issuer: &str,
        encryption_key: &[u8; 32],
    ) -> Result<TotpEnrollment, AppError> {
        // 1. Generate 20-byte random secret
        let mut secret = [0u8; 20];
        ring::rand::SystemRandom::new()
            .fill(&mut secret)
            .map_err(|_| AppError::Internal(anyhow::anyhow!("RNG failed")))?;

        // 2. Base32 encode for user display
        let secret_base32 = base32_encode(&secret);

        // 3. Encrypt for DB storage
        let secret_enc_bytes = crypto::encrypt(&secret, encryption_key)?;
        let secret_enc = base64::engine::general_purpose::STANDARD.encode(&secret_enc_bytes);

        // 4. Build otpauth URI
        let totp_uri = format!(
            "otpauth://totp/{}:{}?secret={}&issuer={}&algorithm=SHA1&digits=6&period=30",
            issuer, user_email, secret_base32, issuer
        );

        Ok(TotpEnrollment {
            secret_enc,
            totp_uri,
            secret_base32,
        })
    }

    /// Verify a TOTP code.
    /// Accepts current time step and +/-1 adjacent (drift tolerance).
    pub fn verify_totp(
        code: &str,
        secret_enc: &str,
        encryption_key: &[u8; 32],
    ) -> Result<bool, AppError> {
        // 1. Decrypt secret
        let secret_enc_bytes = base64::engine::general_purpose::STANDARD
            .decode(secret_enc)
            .map_err(|_| AppError::Internal(anyhow::anyhow!("Invalid base64")))?;
        let secret = crypto::decrypt(&secret_enc_bytes, encryption_key)?;

        // 2. Get current time step
        let time = Utc::now().timestamp() as u64;
        let time_step = time / 30;

        // 3. Check current and +/-1 adjacent time steps (constant-time)
        let mut valid = false;
        for offset in [-1i64, 0, 1] {
            let step = (time_step as i64 + offset) as u64;
            let expected = generate_totp_code(&secret, step);
            if crypto::constant_time_eq(code.as_bytes(), expected.as_bytes()) {
                valid = true;
            }
        }

        Ok(valid)
    }

    /// Generate backup codes (10 codes, 8 chars each).
    pub fn generate_backup_codes() -> Vec<String> {
        (0..10)
            .map(|_| {
                let token = crypto::generate_token();
                token[..8].to_uppercase()
            })
            .collect()
    }

    /// Encrypt backup codes for storage.
    pub fn encrypt_backup_codes(
        codes: &[String],
        encryption_key: &[u8; 32],
    ) -> Result<String, AppError> {
        let json = serde_json::to_vec(codes).map_err(|e| AppError::Internal(e.into()))?;
        let encrypted = crypto::encrypt(&json, encryption_key)?;
        Ok(base64::engine::general_purpose::STANDARD.encode(&encrypted))
    }

    /// Decrypt and check a backup code (removes it from the list if valid).
    /// Returns (is_valid, updated_encrypted_codes).
    pub fn verify_backup_code(
        code: &str,
        encrypted_codes: &str,
        encryption_key: &[u8; 32],
    ) -> Result<(bool, String), AppError> {
        let enc_bytes = base64::engine::general_purpose::STANDARD
            .decode(encrypted_codes)
            .map_err(|_| AppError::Internal(anyhow::anyhow!("Invalid base64")))?;
        let decrypted = crypto::decrypt(&enc_bytes, encryption_key)?;
        let mut codes: Vec<String> =
            serde_json::from_slice(&decrypted).map_err(|e| AppError::Internal(e.into()))?;

        let code_upper = code.to_uppercase();
        if let Some(pos) = codes
            .iter()
            .position(|c| crypto::constant_time_eq(c.as_bytes(), code_upper.as_bytes()))
        {
            codes.remove(pos);
            // Re-encrypt remaining codes
            let new_encrypted = Self::encrypt_backup_codes(&codes, encryption_key)?;
            Ok((true, new_encrypted))
        } else {
            Ok((false, encrypted_codes.to_string()))
        }
    }
}

/// Generate a 6-digit TOTP code for a given time step (RFC 6238).
fn generate_totp_code(secret: &[u8], time_step: u64) -> String {
    let time_bytes = time_step.to_be_bytes();
    let key = hmac::Key::new(hmac::HMAC_SHA1_FOR_LEGACY_USE_ONLY, secret);
    let signature = hmac::sign(&key, &time_bytes);
    let hash = signature.as_ref();

    let offset = (hash[hash.len() - 1] & 0x0F) as usize;
    let code = ((hash[offset] as u32 & 0x7F) << 24)
        | ((hash[offset + 1] as u32) << 16)
        | ((hash[offset + 2] as u32) << 8)
        | (hash[offset + 3] as u32);

    format!("{:06}", code % 1_000_000)
}

/// Base32 encode (RFC 4648, no padding).
fn base32_encode(data: &[u8]) -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
    let mut result = String::new();
    let mut buffer: u64 = 0;
    let mut bits_in_buffer = 0;

    for &byte in data {
        buffer = (buffer << 8) | byte as u64;
        bits_in_buffer += 8;
        while bits_in_buffer >= 5 {
            bits_in_buffer -= 5;
            let index = ((buffer >> bits_in_buffer) & 0x1F) as usize;
            result.push(ALPHABET[index] as char);
        }
    }
    if bits_in_buffer > 0 {
        let index = ((buffer << (5 - bits_in_buffer)) & 0x1F) as usize;
        result.push(ALPHABET[index] as char);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_key() -> [u8; 32] {
        crypto::generate_encryption_key()
    }

    #[test]
    fn enroll_totp_produces_valid_uri() {
        let enrollment = MfaService::enroll_totp("user@test.com", "Nucleus", &test_key()).unwrap();
        assert!(enrollment.totp_uri.starts_with("otpauth://totp/"));
        assert!(enrollment.totp_uri.contains("Nucleus"));
        assert!(enrollment.totp_uri.contains("user@test.com"));
        assert!(!enrollment.secret_base32.is_empty());
        assert!(!enrollment.secret_enc.is_empty());
    }

    #[test]
    fn verify_totp_with_current_code() {
        let key = test_key();
        let enrollment = MfaService::enroll_totp("user@test.com", "Nucleus", &key).unwrap();

        // Decrypt to get raw secret for manual code generation
        let enc_bytes = base64::engine::general_purpose::STANDARD
            .decode(&enrollment.secret_enc)
            .unwrap();
        let secret = crypto::decrypt(&enc_bytes, &key).unwrap();

        // Generate current code
        let time_step = Utc::now().timestamp() as u64 / 30;
        let code = generate_totp_code(&secret, time_step);

        let valid = MfaService::verify_totp(&code, &enrollment.secret_enc, &key).unwrap();
        assert!(valid);
    }

    #[test]
    fn verify_totp_rejects_wrong_code() {
        let key = test_key();
        let enrollment = MfaService::enroll_totp("user@test.com", "Nucleus", &key).unwrap();
        let valid = MfaService::verify_totp("000000", &enrollment.secret_enc, &key).unwrap();
        assert!(!valid);
    }

    #[test]
    fn generate_backup_codes_produces_10_codes() {
        let codes = MfaService::generate_backup_codes();
        assert_eq!(codes.len(), 10);
        for code in &codes {
            assert_eq!(code.len(), 8);
        }
    }

    #[test]
    fn backup_codes_are_unique() {
        let codes = MfaService::generate_backup_codes();
        let mut unique = codes.clone();
        unique.sort();
        unique.dedup();
        assert_eq!(unique.len(), 10);
    }

    #[test]
    fn encrypt_decrypt_backup_codes_roundtrip() {
        let key = test_key();
        let codes = MfaService::generate_backup_codes();
        let encrypted = MfaService::encrypt_backup_codes(&codes, &key).unwrap();

        // Verify by using a valid code
        let (valid, remaining) =
            MfaService::verify_backup_code(&codes[0], &encrypted, &key).unwrap();
        assert!(valid);

        // Using same code again should fail
        let (valid2, _) = MfaService::verify_backup_code(&codes[0], &remaining, &key).unwrap();
        assert!(!valid2);
    }

    #[test]
    fn backup_code_verification_case_insensitive() {
        let key = test_key();
        let codes = MfaService::generate_backup_codes();
        let encrypted = MfaService::encrypt_backup_codes(&codes, &key).unwrap();
        let lower = codes[0].to_lowercase();
        let (valid, _) = MfaService::verify_backup_code(&lower, &encrypted, &key).unwrap();
        assert!(valid);
    }

    #[test]
    fn backup_code_single_use() {
        let key = test_key();
        let codes = MfaService::generate_backup_codes();
        let encrypted = MfaService::encrypt_backup_codes(&codes, &key).unwrap();

        // Use first code
        let (valid, remaining) =
            MfaService::verify_backup_code(&codes[0], &encrypted, &key).unwrap();
        assert!(valid);

        // Try first code again
        let (valid2, _) = MfaService::verify_backup_code(&codes[0], &remaining, &key).unwrap();
        assert!(!valid2);

        // Second code should still work
        let (valid3, _) = MfaService::verify_backup_code(&codes[1], &remaining, &key).unwrap();
        assert!(valid3);
    }

    #[test]
    fn totp_code_is_6_digits() {
        let secret = [0u8; 20]; // all zeros for reproducibility
        let code = generate_totp_code(&secret, 1);
        assert_eq!(code.len(), 6);
        assert!(code.chars().all(|c| c.is_ascii_digit()));
    }

    #[test]
    fn base32_encode_works() {
        let result = base32_encode(b"test");
        assert!(!result.is_empty());
        assert!(result
            .chars()
            .all(|c| "ABCDEFGHIJKLMNOPQRSTUVWXYZ234567".contains(c)));
    }
}
