use crate::core::crypto;
use crate::core::error::{AppError, AuthError};
use chrono::{DateTime, Duration, Utc};

pub struct OtpConfig {
    pub digits: u32,
    pub ttl_secs: u64,
    pub max_attempts: u32,
}

impl Default for OtpConfig {
    fn default() -> Self {
        Self {
            digits: 6,
            ttl_secs: 300,
            max_attempts: 3,
        }
    }
}

pub struct OtpService;

/// Generated OTP data (to be stored in Redis)
pub struct OtpGenerated {
    pub code: String,
    pub code_hash: String,
    pub expires_at: DateTime<Utc>,
    pub max_attempts: u32,
}

impl OtpService {
    /// Generate a new OTP code
    pub fn generate(config: &OtpConfig) -> OtpGenerated {
        let code = crypto::generate_otp_code(config.digits);
        let code_hash = crypto::generate_token_hash(&code);
        let expires_at = Utc::now() + Duration::seconds(config.ttl_secs as i64);

        OtpGenerated {
            code,
            code_hash,
            expires_at,
            max_attempts: config.max_attempts,
        }
    }

    /// Verify an OTP code.
    /// Returns Err if: wrong code, expired, or max attempts exceeded.
    pub fn verify(
        provided_code: &str,
        stored_hash: &str,
        expires_at: &DateTime<Utc>,
        attempts: u32,
        max_attempts: u32,
    ) -> Result<(), AppError> {
        // 1. Check max attempts
        if attempts >= max_attempts {
            return Err(AppError::Auth(AuthError::OtpMaxAttempts));
        }

        // 2. Check expiry
        if Utc::now() > *expires_at {
            return Err(AppError::Auth(AuthError::OtpExpired));
        }

        // 3. Verify code hash (constant-time to prevent timing attacks)
        let hash = crypto::generate_token_hash(provided_code);
        if !crypto::constant_time_eq(hash.as_bytes(), stored_hash.as_bytes()) {
            return Err(AppError::Auth(AuthError::InvalidCredentials));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_otp_correct_digits() {
        let config = OtpConfig::default();
        let otp = OtpService::generate(&config);
        assert_eq!(otp.code.len(), 6);
        assert!(otp.code.chars().all(|c| c.is_ascii_digit()));
    }

    #[test]
    fn generate_otp_unique() {
        let config = OtpConfig::default();
        let o1 = OtpService::generate(&config);
        let o2 = OtpService::generate(&config);
        // Not guaranteed unique but hash should differ
        assert_ne!(o1.code_hash, o2.code_hash);
    }

    #[test]
    fn verify_correct_code() {
        let config = OtpConfig::default();
        let otp = OtpService::generate(&config);
        let result = OtpService::verify(
            &otp.code,
            &otp.code_hash,
            &otp.expires_at,
            0,
            otp.max_attempts,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn verify_wrong_code() {
        let config = OtpConfig::default();
        let otp = OtpService::generate(&config);
        let result = OtpService::verify(
            "000000",
            &otp.code_hash,
            &otp.expires_at,
            0,
            otp.max_attempts,
        );
        assert!(matches!(
            result,
            Err(AppError::Auth(AuthError::InvalidCredentials))
        ));
    }

    #[test]
    fn verify_expired_code() {
        let config = OtpConfig::default();
        let otp = OtpService::generate(&config);
        let expired = Utc::now() - Duration::minutes(1);
        let result = OtpService::verify(&otp.code, &otp.code_hash, &expired, 0, otp.max_attempts);
        assert!(matches!(result, Err(AppError::Auth(AuthError::OtpExpired))));
    }

    #[test]
    fn verify_max_attempts_exceeded() {
        let config = OtpConfig::default();
        let otp = OtpService::generate(&config);
        let result = OtpService::verify(&otp.code, &otp.code_hash, &otp.expires_at, 3, 3);
        assert!(matches!(
            result,
            Err(AppError::Auth(AuthError::OtpMaxAttempts))
        ));
    }

    #[test]
    fn custom_config() {
        let config = OtpConfig {
            digits: 8,
            ttl_secs: 600,
            max_attempts: 5,
        };
        let otp = OtpService::generate(&config);
        assert_eq!(otp.code.len(), 8);
        assert_eq!(otp.max_attempts, 5);
    }

    #[test]
    fn expires_in_configured_time() {
        let config = OtpConfig {
            digits: 6,
            ttl_secs: 120,
            max_attempts: 3,
        };
        let otp = OtpService::generate(&config);
        let expected = Utc::now() + Duration::seconds(120);
        let diff = (otp.expires_at - expected).num_seconds().abs();
        assert!(diff < 2);
    }
}
