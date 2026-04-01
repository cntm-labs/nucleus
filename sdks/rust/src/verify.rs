use crate::claims::NucleusClaims;
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use serde::Deserialize;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Errors produced by the Nucleus SDK.
#[derive(Debug, thiserror::Error)]
pub enum NucleusError {
    #[error("nucleus: failed to fetch JWKS: {0}")]
    JwksFetch(String),

    #[error("nucleus: no matching key found in JWKS for kid `{0}`")]
    KeyNotFound(String),

    #[error("nucleus: invalid token: {0}")]
    InvalidToken(String),

    #[error("nucleus: API error {status}: {message}")]
    Api { status: u16, message: String },

    #[error("nucleus: HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),
}

// ── JWKS types ───────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct JwksResponse {
    keys: Vec<Jwk>,
}

#[derive(Debug, Clone, Deserialize)]
struct Jwk {
    kid: Option<String>,
    kty: String,
    n: Option<String>,
    e: Option<String>,
}

// ── Cached verifier ──────────────────────────────────────────────────────────

struct CachedKeys {
    keys: HashMap<String, DecodingKey>,
    fetched_at: Instant,
}

pub struct JwksVerifier {
    jwks_url: String,
    ttl: Duration,
    cache: RwLock<Option<CachedKeys>>,
    http: reqwest::Client,
}

impl JwksVerifier {
    pub fn new(base_url: &str, ttl_secs: u64) -> Self {
        let jwks_url = format!("{}/.well-known/jwks.json", base_url.trim_end_matches('/'));
        Self {
            jwks_url,
            ttl: Duration::from_secs(ttl_secs),
            cache: RwLock::new(None),
            http: reqwest::Client::new(),
        }
    }

    /// Verify a JWT token against the cached JWKS and return parsed claims.
    pub async fn verify(&self, token: &str) -> Result<NucleusClaims, NucleusError> {
        let header = decode_header(token)
            .map_err(|e| NucleusError::InvalidToken(e.to_string()))?;
        let kid = header
            .kid
            .ok_or_else(|| NucleusError::InvalidToken("token header missing `kid`".into()))?;

        let key = self.get_key(&kid).await?;

        let mut validation = Validation::new(Algorithm::RS256);
        // Nucleus tokens carry project-specific audiences; disable default aud
        // validation so callers can check it themselves if needed.
        validation.validate_aud = false;

        let data = decode::<NucleusClaims>(token, &key, &validation)
            .map_err(|e| NucleusError::InvalidToken(e.to_string()))?;

        Ok(data.claims)
    }

    // ── Internal helpers ─────────────────────────────────────────────────

    async fn get_key(&self, kid: &str) -> Result<DecodingKey, NucleusError> {
        // Fast path — read lock.
        {
            let guard = self.cache.read().await;
            if let Some(cached) = guard.as_ref() {
                if cached.fetched_at.elapsed() < self.ttl {
                    if let Some(key) = cached.keys.get(kid) {
                        return Ok(key.clone());
                    }
                }
            }
        }

        // Slow path — fetch fresh JWKS under write lock.
        self.refresh_and_get(kid).await
    }

    async fn refresh_and_get(&self, kid: &str) -> Result<DecodingKey, NucleusError> {
        let mut guard = self.cache.write().await;

        // Double-check after acquiring write lock.
        if let Some(cached) = guard.as_ref() {
            if cached.fetched_at.elapsed() < self.ttl {
                if let Some(key) = cached.keys.get(kid) {
                    return Ok(key.clone());
                }
            }
        }

        let resp: JwksResponse = self
            .http
            .get(&self.jwks_url)
            .send()
            .await?
            .error_for_status()
            .map_err(|e| NucleusError::JwksFetch(e.to_string()))?
            .json()
            .await
            .map_err(|e| NucleusError::JwksFetch(e.to_string()))?;

        let mut keys = HashMap::new();
        for jwk in &resp.keys {
            if jwk.kty != "RSA" {
                continue;
            }
            if let (Some(n), Some(e), Some(k)) = (&jwk.n, &jwk.e, &jwk.kid) {
                if let Ok(dk) = DecodingKey::from_rsa_components(n, e) {
                    keys.insert(k.clone(), dk);
                }
            }
        }

        let key = keys
            .get(kid)
            .ok_or_else(|| NucleusError::KeyNotFound(kid.to_string()))?
            .clone();

        *guard = Some(CachedKeys {
            keys,
            fetched_at: Instant::now(),
        });

        Ok(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::claims::NucleusClaims;

    #[test]
    fn verifier_creation_stores_jwks_url() {
        let verifier = JwksVerifier::new("https://api.test.com", 3600);
        assert_eq!(verifier.jwks_url, "https://api.test.com/.well-known/jwks.json");
    }

    #[test]
    fn verifier_trims_trailing_slash() {
        let verifier = JwksVerifier::new("https://api.test.com/", 3600);
        assert_eq!(verifier.jwks_url, "https://api.test.com/.well-known/jwks.json");
    }

    #[test]
    fn verifier_stores_ttl() {
        let verifier = JwksVerifier::new("https://api.test.com", 7200);
        assert_eq!(verifier.ttl, Duration::from_secs(7200));
    }

    #[tokio::test]
    async fn verify_invalid_token_returns_error() {
        let verifier = JwksVerifier::new("https://api.test.com", 3600);
        let result = verifier.verify("not.a.valid.token").await;
        assert!(result.is_err());
        match result {
            Err(NucleusError::InvalidToken(_)) => {}
            other => panic!("expected InvalidToken error, got: {:?}", other),
        }
    }

    #[tokio::test]
    async fn verify_empty_token_returns_error() {
        let verifier = JwksVerifier::new("https://api.test.com", 3600);
        let result = verifier.verify("").await;
        assert!(result.is_err());
    }

    #[test]
    fn claims_serde_roundtrip() {
        let json = serde_json::json!({
            "sub": "user_123",
            "iss": "https://api.test.com",
            "aud": "project_456",
            "exp": 1700000000u64,
            "iat": 1699996400u64,
            "jti": "jwt_abc",
            "email": "test@example.com",
            "first_name": "Test",
            "last_name": "User",
            "org_id": "org_1",
            "org_role": "admin",
            "org_permissions": ["read", "write"]
        });

        let claims: NucleusClaims = serde_json::from_value(json).unwrap();
        assert_eq!(claims.user_id(), "user_123");
        assert_eq!(claims.sub, "user_123");
        assert_eq!(claims.aud, "project_456");
        assert_eq!(claims.email, Some("test@example.com".to_string()));
        assert_eq!(claims.first_name, Some("Test".to_string()));
        assert_eq!(claims.org_id, Some("org_1".to_string()));
        assert_eq!(claims.org_role, Some("admin".to_string()));
        assert_eq!(
            claims.org_permissions,
            Some(vec!["read".to_string(), "write".to_string()])
        );

        // Roundtrip
        let serialized = serde_json::to_value(&claims).unwrap();
        assert_eq!(serialized["sub"], "user_123");
    }

    #[test]
    fn claims_missing_optional_fields() {
        let json = serde_json::json!({
            "sub": "user_1",
            "iss": "https://test.com",
            "aud": "proj_1",
            "exp": 1700000000u64,
            "iat": 1699996400u64
        });

        let claims: NucleusClaims = serde_json::from_value(json).unwrap();
        assert_eq!(claims.user_id(), "user_1");
        assert!(claims.email.is_none());
        assert!(claims.org_id.is_none());
        assert!(claims.org_permissions.is_none());
        assert!(claims.metadata.is_none());
    }

    #[test]
    fn nucleus_error_display() {
        let err = NucleusError::InvalidToken("test error".into());
        assert!(err.to_string().contains("invalid token"));

        let err = NucleusError::KeyNotFound("kid-123".into());
        assert!(err.to_string().contains("kid-123"));

        let err = NucleusError::JwksFetch("timeout".into());
        assert!(err.to_string().contains("JWKS"));
    }
}
