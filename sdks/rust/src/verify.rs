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
