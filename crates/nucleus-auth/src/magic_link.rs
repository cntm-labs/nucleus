use chrono::{DateTime, Duration, Utc};
use nucleus_core::crypto;
use nucleus_core::error::{AppError, AuthError};

pub struct MagicLinkService;

/// Result of generating a magic link
pub struct MagicLinkGenerated {
    /// The token to include in the magic link URL (send to user)
    pub token: String,
    /// The hash stored in the database
    pub token_hash: String,
    /// When the token expires
    pub expires_at: DateTime<Utc>,
}

impl MagicLinkService {
    /// Generate a magic link token
    /// Token lifetime: 15 minutes
    pub fn generate() -> MagicLinkGenerated {
        let token = crypto::generate_token();
        let token_hash = crypto::generate_token_hash(&token);
        let expires_at = Utc::now() + Duration::minutes(15);

        MagicLinkGenerated {
            token,
            token_hash,
            expires_at,
        }
    }

    /// Verify a magic link token
    /// Checks: hash matches, not expired, not already used
    pub fn verify_token(
        provided_token: &str,
        stored_hash: &str,
        expires_at: &DateTime<Utc>,
        used_at: &Option<DateTime<Utc>>,
    ) -> Result<(), AppError> {
        // 1. Check not already used
        if used_at.is_some() {
            return Err(AppError::Auth(AuthError::MagicLinkExpired));
        }

        // 2. Check not expired
        if Utc::now() > *expires_at {
            return Err(AppError::Auth(AuthError::MagicLinkExpired));
        }

        // 3. Verify hash matches (constant-time to prevent timing attacks)
        let hash = crypto::generate_token_hash(provided_token);
        if !crypto::constant_time_eq(hash.as_bytes(), stored_hash.as_bytes()) {
            return Err(AppError::Auth(AuthError::TokenInvalid));
        }

        Ok(())
    }

    /// Build the magic link URL.
    /// SECURITY: validates redirect_url to prevent open redirect attacks.
    pub fn build_url(
        base_url: &str,
        token: &str,
        redirect_url: &str,
        allowed_domains: &[&str],
    ) -> Result<String, AppError> {
        // Validate redirect_url
        let parsed = url::Url::parse(redirect_url)
            .map_err(|_| AppError::Auth(AuthError::InvalidRedirectUrl))?;

        // Only allow https (or http for localhost in dev)
        match parsed.scheme() {
            "https" => {}
            "http"
                if parsed.host_str() == Some("localhost")
                    || parsed.host_str() == Some("127.0.0.1") => {}
            _ => return Err(AppError::Auth(AuthError::InvalidRedirectUrl)),
        }

        // Validate domain is in allowlist
        let host = parsed.host_str().unwrap_or("");
        let domain_allowed = allowed_domains.is_empty()
            || allowed_domains
                .iter()
                .any(|d| host == *d || host.ends_with(&format!(".{}", d)));
        if !domain_allowed {
            return Err(AppError::Auth(AuthError::InvalidRedirectUrl));
        }

        Ok(format!(
            "{}/api/v1/auth/magic-link/verify?token={}&redirect_url={}",
            base_url,
            urlencoding::encode(token),
            urlencoding::encode(redirect_url),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_produces_token_and_hash() {
        let result = MagicLinkService::generate();
        assert!(!result.token.is_empty());
        assert!(!result.token_hash.is_empty());
        assert_ne!(result.token, result.token_hash);
    }

    #[test]
    fn generate_token_is_unique() {
        let r1 = MagicLinkService::generate();
        let r2 = MagicLinkService::generate();
        assert_ne!(r1.token, r2.token);
    }

    #[test]
    fn verify_valid_token() {
        let generated = MagicLinkService::generate();
        let result = MagicLinkService::verify_token(
            &generated.token,
            &generated.token_hash,
            &generated.expires_at,
            &None,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn verify_rejects_wrong_token() {
        let generated = MagicLinkService::generate();
        let result = MagicLinkService::verify_token(
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
        let generated = MagicLinkService::generate();
        let expired = Utc::now() - Duration::minutes(1);
        let result = MagicLinkService::verify_token(
            &generated.token,
            &generated.token_hash,
            &expired,
            &None,
        );
        assert!(matches!(
            result,
            Err(AppError::Auth(AuthError::MagicLinkExpired))
        ));
    }

    #[test]
    fn verify_rejects_already_used_token() {
        let generated = MagicLinkService::generate();
        let used = Some(Utc::now());
        let result = MagicLinkService::verify_token(
            &generated.token,
            &generated.token_hash,
            &generated.expires_at,
            &used,
        );
        assert!(matches!(
            result,
            Err(AppError::Auth(AuthError::MagicLinkExpired))
        ));
    }

    #[test]
    fn build_url_includes_token_and_redirect() {
        let url = MagicLinkService::build_url(
            "https://nucleus.example.com",
            "my_token_123",
            "https://app.example.com/dashboard",
            &["example.com"],
        )
        .unwrap();
        assert!(url.contains("my_token_123"));
        assert!(url.contains("redirect_url="));
        assert!(url.starts_with("https://nucleus.example.com"));
    }

    #[test]
    fn build_url_rejects_http_non_localhost() {
        let result = MagicLinkService::build_url(
            "https://nucleus.example.com",
            "token",
            "http://evil.com/steal",
            &["example.com"],
        );
        assert!(result.is_err());
    }

    #[test]
    fn build_url_rejects_disallowed_domain() {
        let result = MagicLinkService::build_url(
            "https://nucleus.example.com",
            "token",
            "https://evil.com/phish",
            &["example.com"],
        );
        assert!(result.is_err());
    }

    #[test]
    fn build_url_allows_localhost_http() {
        let url = MagicLinkService::build_url(
            "https://nucleus.example.com",
            "token",
            "http://localhost:3000/callback",
            &[],
        );
        assert!(url.is_ok());
    }

    #[test]
    fn expires_in_15_minutes() {
        let generated = MagicLinkService::generate();
        let expected = Utc::now() + Duration::minutes(15);
        let diff = (generated.expires_at - expected).num_seconds().abs();
        assert!(diff < 2); // within 2 seconds tolerance
    }
}
