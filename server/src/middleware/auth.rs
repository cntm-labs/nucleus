use std::sync::Arc;

use crate::auth::jwt::{JwtService, NucleusClaims};
use crate::core::error::{ApiError, AppError, AuthError};
use crate::core::types::ProjectId;
use crate::session::SessionService;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use sha2::{Digest, Sha256};

// ---------------------------------------------------------------------------
// Public key extension (set by upstream middleware per-project)
// ---------------------------------------------------------------------------

/// Holds the PEM-encoded RSA public key for the current project.
/// A middleware layer earlier in the stack is responsible for looking up
/// the project (from the URL or Host header) and inserting this into
/// request extensions.
#[derive(Clone, Debug)]
pub struct PublicKeyPem(pub Vec<u8>);

// ---------------------------------------------------------------------------
// JwtAuth extractor
// ---------------------------------------------------------------------------

/// Extracts and verifies a JWT from the `Authorization: Bearer <token>` header.
///
/// The token is verified against the RSA public key that a prior middleware
/// placed in [`PublicKeyPem`] request extensions.  Only RS256 is accepted
/// (enforced by [`JwtService::verify`]).
pub struct JwtAuth(pub NucleusClaims);

impl<S> FromRequestParts<S> for JwtAuth
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // 1. Extract Authorization header
        let auth_header = parts
            .headers
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .ok_or(AppError::Auth(AuthError::TokenInvalid))?;

        // 2. Must be Bearer scheme
        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or(AppError::Auth(AuthError::TokenInvalid))?;

        // 3. Get the project public key from request extensions
        let public_key = parts
            .extensions
            .get::<PublicKeyPem>()
            .ok_or(AppError::Auth(AuthError::TokenInvalid))?;

        // 4. Get the expected project ID for audience validation
        let project_id = parts
            .extensions
            .get::<ProjectId>()
            .ok_or(AppError::Auth(AuthError::TokenInvalid))?;

        // 5. Verify JWT (RS256 only, audience = project_id)
        let claims = JwtService::verify(token, &public_key.0, &project_id.to_string())?;

        // 6. Check JWT revocation list
        if let Some(session_service) = parts.extensions.get::<Arc<SessionService>>() {
            if session_service.is_jwt_revoked(&claims.jti).await? {
                return Err(AppError::Auth(AuthError::TokenRevoked));
            }
        }

        Ok(JwtAuth(claims))
    }
}

// ---------------------------------------------------------------------------
// ApiKeyAuth extractor
// ---------------------------------------------------------------------------

/// Result of a successful API-key authentication.
pub struct ApiKeyAuth {
    pub project_id: ProjectId,
    pub key_type: String,
    pub scopes: Vec<String>,
}

/// Trait that the API-key extractor uses to look up and update keys.
/// Implemented by the real Postgres repo; tests supply a mock.
#[async_trait::async_trait]
pub trait ApiKeyLookup: Send + Sync {
    async fn find_by_prefix(
        &self,
        prefix: &str,
    ) -> Result<Option<crate::db::repos::api_key_repo::ApiKey>, AppError>;

    async fn update_last_used(&self, id: &crate::core::types::ApiKeyId) -> Result<(), AppError>;
}

/// Authenticate an API key against a [`ApiKeyLookup`] backend.
///
/// Extracts the bearer token, validates it as an API key, looks it up by
/// prefix, verifies the hash, and returns an [`ApiKeyAuth`] on success.
pub async fn authenticate_api_key(
    auth_header: &str,
    lookup: &dyn ApiKeyLookup,
) -> Result<ApiKeyAuth, AppError> {
    let raw_token =
        extract_bearer_token(auth_header).ok_or(AppError::Auth(AuthError::TokenInvalid))?;

    if !is_api_key_format(raw_token) {
        return Err(AppError::Auth(AuthError::TokenInvalid));
    }

    let prefix =
        extract_api_key_prefix(raw_token).ok_or(AppError::Auth(AuthError::TokenInvalid))?;

    let key_record = lookup
        .find_by_prefix(prefix)
        .await?
        .ok_or(AppError::Auth(AuthError::TokenInvalid))?;

    validate_api_key(&key_record)?;

    let provided_hash = hash_api_key(raw_token);
    if !verify_key_hash(&provided_hash, &key_record.key_hash) {
        return Err(AppError::Auth(AuthError::TokenInvalid));
    }

    let key_type = detect_key_type(raw_token).unwrap_or("unknown").to_string();

    lookup.update_last_used(&key_record.id).await?;

    Ok(ApiKeyAuth {
        project_id: key_record.project_id,
        key_type,
        scopes: key_record.scopes,
    })
}

impl<S> FromRequestParts<S> for ApiKeyAuth
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .ok_or(AppError::Auth(AuthError::TokenInvalid))?;

        let lookup = parts
            .extensions
            .get::<std::sync::Arc<dyn ApiKeyLookup>>()
            .ok_or(AppError::Auth(AuthError::TokenInvalid))?;

        authenticate_api_key(auth_header, lookup.as_ref()).await
    }
}

// ---------------------------------------------------------------------------
// Helpers (kept public for unit testing)
// ---------------------------------------------------------------------------

/// Extract the bearer token from an Authorization header value.
pub fn extract_bearer_token(header_value: &str) -> Option<&str> {
    header_value.strip_prefix("Bearer ")
}

/// Detect whether a raw token looks like an API key (pk_ or sk_ prefix).
pub fn is_api_key_format(token: &str) -> bool {
    token.starts_with("pk_") || token.starts_with("sk_")
}

/// Extract the prefix portion of an API key.
/// Convention: everything up to and including the 12th character, e.g.
///   `pk_live_abc123xyz...` → `pk_live_abc12`
/// This must match the prefix stored in the database when the key was created.
pub fn extract_api_key_prefix(api_key: &str) -> Option<&str> {
    if api_key.len() >= 12 {
        Some(&api_key[..12])
    } else {
        None
    }
}

/// Determine the key type from the raw API key string.
pub fn detect_key_type(api_key: &str) -> Option<&'static str> {
    if api_key.starts_with("pk_") {
        Some("publishable")
    } else if api_key.starts_with("sk_") {
        Some("secret")
    } else {
        None
    }
}

/// Compute the SHA-256 hex digest of an API key (for comparison with stored hash).
pub fn hash_api_key(api_key: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(api_key.as_bytes());
    hex::encode(hasher.finalize())
}

/// Constant-time comparison of two hex-encoded hashes.
pub fn verify_key_hash(provided_hash: &str, stored_hash: &str) -> bool {
    crate::core::crypto::constant_time_eq(provided_hash.as_bytes(), stored_hash.as_bytes())
}

/// Validate an [`ApiKey`] record: not revoked, not expired.
pub fn validate_api_key(key: &crate::db::repos::api_key_repo::ApiKey) -> Result<(), AppError> {
    if key.revoked_at.is_some() {
        return Err(AppError::Api(ApiError::KeyRevoked));
    }
    if let Some(expires_at) = key.expires_at {
        if expires_at < chrono::Utc::now() {
            return Err(AppError::Api(ApiError::KeyExpired));
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- Bearer token extraction --

    #[test]
    fn extract_bearer_token_from_header() {
        let header = "Bearer eyJhbGciOiJSUzI1NiJ9.payload.signature";
        let token = extract_bearer_token(header);
        assert_eq!(token, Some("eyJhbGciOiJSUzI1NiJ9.payload.signature"));
    }

    #[test]
    fn reject_missing_authorization_header() {
        // No header at all → None
        let token = extract_bearer_token("");
        assert!(token.is_none());
    }

    #[test]
    fn reject_non_bearer_scheme() {
        let token = extract_bearer_token("Basic dXNlcjpwYXNz");
        assert!(token.is_none());
    }

    // -- API key format detection --

    #[test]
    fn api_key_format_detection() {
        assert!(is_api_key_format("pk_live_abc123xyz"));
        assert!(is_api_key_format("sk_live_abc123xyz"));
        assert!(!is_api_key_format("eyJhbGciOiJSUzI1NiJ9"));
        assert!(!is_api_key_format("Basic abc"));
    }

    // -- Prefix parsing --

    #[test]
    fn api_key_prefix_parsing() {
        let key = "pk_live_abc123xyz_rest_of_key";
        let prefix = extract_api_key_prefix(key);
        assert_eq!(prefix, Some("pk_live_abc1"));
    }

    #[test]
    fn api_key_prefix_too_short() {
        let prefix = extract_api_key_prefix("pk_");
        assert!(prefix.is_none());
    }

    // -- Key type detection --

    #[test]
    fn api_key_type_detection() {
        assert_eq!(detect_key_type("pk_live_something"), Some("publishable"));
        assert_eq!(detect_key_type("sk_live_something"), Some("secret"));
        assert_eq!(detect_key_type("unknown_prefix"), None);
    }

    // -- Hash verification --

    #[test]
    fn hash_api_key_is_deterministic() {
        let key = "sk_live_test_key_12345";
        assert_eq!(hash_api_key(key), hash_api_key(key));
    }

    #[test]
    fn hash_api_key_differs_for_different_keys() {
        assert_ne!(hash_api_key("pk_live_aaa"), hash_api_key("pk_live_bbb"));
    }

    #[test]
    fn verify_key_hash_matches_correct() {
        let key = "sk_live_abc123";
        let hash = hash_api_key(key);
        assert!(verify_key_hash(&hash, &hash));
    }

    #[test]
    fn verify_key_hash_rejects_mismatch() {
        let hash_a = hash_api_key("sk_live_aaa");
        let hash_b = hash_api_key("sk_live_bbb");
        assert!(!verify_key_hash(&hash_a, &hash_b));
    }

    // -- API key validation --

    #[test]
    fn validate_api_key_rejects_revoked() {
        let key = crate::db::repos::api_key_repo::ApiKey {
            id: crate::core::types::ApiKeyId::new(),
            project_id: crate::core::types::ProjectId::new(),
            key_type: "secret".to_string(),
            key_hash: "abc".to_string(),
            key_prefix: "sk_live_abc1".to_string(),
            environment: "live".to_string(),
            label: None,
            scopes: vec![],
            rate_limit: None,
            last_used_at: None,
            expires_at: None,
            created_at: chrono::Utc::now(),
            revoked_at: Some(chrono::Utc::now()),
        };
        let result = validate_api_key(&key);
        assert!(matches!(result, Err(AppError::Api(ApiError::KeyRevoked))));
    }

    #[test]
    fn validate_api_key_rejects_expired() {
        let key = crate::db::repos::api_key_repo::ApiKey {
            id: crate::core::types::ApiKeyId::new(),
            project_id: crate::core::types::ProjectId::new(),
            key_type: "secret".to_string(),
            key_hash: "abc".to_string(),
            key_prefix: "sk_live_abc1".to_string(),
            environment: "live".to_string(),
            label: None,
            scopes: vec![],
            rate_limit: None,
            last_used_at: None,
            expires_at: Some(chrono::Utc::now() - chrono::Duration::hours(1)),
            created_at: chrono::Utc::now(),
            revoked_at: None,
        };
        let result = validate_api_key(&key);
        assert!(matches!(result, Err(AppError::Api(ApiError::KeyExpired))));
    }

    #[test]
    fn validate_api_key_accepts_valid() {
        let key = crate::db::repos::api_key_repo::ApiKey {
            id: crate::core::types::ApiKeyId::new(),
            project_id: crate::core::types::ProjectId::new(),
            key_type: "secret".to_string(),
            key_hash: "abc".to_string(),
            key_prefix: "sk_live_abc1".to_string(),
            environment: "live".to_string(),
            label: None,
            scopes: vec!["admin:read".to_string()],
            rate_limit: None,
            last_used_at: None,
            expires_at: Some(chrono::Utc::now() + chrono::Duration::hours(24)),
            created_at: chrono::Utc::now(),
            revoked_at: None,
        };
        assert!(validate_api_key(&key).is_ok());
    }
}
