use crate::core::crypto;
use crate::core::error::{AppError, AuthError};
use chrono::{DateTime, Duration, Utc};

pub struct PasswordResetService;

pub struct PasswordResetGenerated {
    pub token: String,
    pub token_hash: String,
    pub expires_at: DateTime<Utc>,
}

impl PasswordResetService {
    /// Generate a password reset token (1 hour expiry)
    pub fn generate() -> PasswordResetGenerated {
        let token = crypto::generate_token();
        let token_hash = crypto::generate_token_hash(&token);
        let expires_at = Utc::now() + Duration::hours(1);

        PasswordResetGenerated {
            token,
            token_hash,
            expires_at,
        }
    }

    /// Verify a password reset token
    pub fn verify_token(
        provided_token: &str,
        stored_hash: &str,
        expires_at: &DateTime<Utc>,
        used_at: &Option<DateTime<Utc>>,
    ) -> Result<(), AppError> {
        // 1. Check not already used
        if used_at.is_some() {
            return Err(AppError::Auth(AuthError::TokenInvalid));
        }

        // 2. Check not expired
        if Utc::now() > *expires_at {
            return Err(AppError::Auth(AuthError::TokenExpired));
        }

        // 3. Verify hash matches (constant-time to prevent timing attacks)
        let hash = crypto::generate_token_hash(provided_token);
        if !crypto::constant_time_eq(hash.as_bytes(), stored_hash.as_bytes()) {
            return Err(AppError::Auth(AuthError::TokenInvalid));
        }

        Ok(())
    }

    /// Build the password reset URL
    pub fn build_url(base_url: &str, token: &str) -> String {
        format!("{}/reset-password?token={}", base_url, token)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_produces_token_and_hash() {
        let result = PasswordResetService::generate();
        assert!(!result.token.is_empty());
        assert!(!result.token_hash.is_empty());
        assert_ne!(result.token, result.token_hash);
    }

    #[test]
    fn generate_unique_tokens() {
        let r1 = PasswordResetService::generate();
        let r2 = PasswordResetService::generate();
        assert_ne!(r1.token, r2.token);
    }

    #[test]
    fn verify_valid_token() {
        let generated = PasswordResetService::generate();
        let result = PasswordResetService::verify_token(
            &generated.token,
            &generated.token_hash,
            &generated.expires_at,
            &None,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn verify_rejects_wrong_token() {
        let generated = PasswordResetService::generate();
        let result = PasswordResetService::verify_token(
            "wrong_token",
            &generated.token_hash,
            &generated.expires_at,
            &None,
        );
        assert!(matches!(
            result,
            Err(AppError::Auth(AuthError::TokenInvalid))
        ));
    }

    #[test]
    fn verify_rejects_expired_token() {
        let generated = PasswordResetService::generate();
        let expired = Utc::now() - Duration::minutes(1);
        let result = PasswordResetService::verify_token(
            &generated.token,
            &generated.token_hash,
            &expired,
            &None,
        );
        assert!(matches!(
            result,
            Err(AppError::Auth(AuthError::TokenExpired))
        ));
    }

    #[test]
    fn verify_rejects_used_token() {
        let generated = PasswordResetService::generate();
        let used = Some(Utc::now());
        let result = PasswordResetService::verify_token(
            &generated.token,
            &generated.token_hash,
            &generated.expires_at,
            &used,
        );
        assert!(matches!(
            result,
            Err(AppError::Auth(AuthError::TokenInvalid))
        ));
    }

    #[test]
    fn expires_in_1_hour() {
        let generated = PasswordResetService::generate();
        let expected = Utc::now() + Duration::hours(1);
        let diff = (generated.expires_at - expected).num_seconds().abs();
        assert!(diff < 2);
    }

    #[test]
    fn build_url_contains_token() {
        let url = PasswordResetService::build_url("https://app.example.com", "my_reset_token");
        assert_eq!(
            url,
            "https://app.example.com/reset-password?token=my_reset_token"
        );
    }
}
