use chrono::{Duration, Utc};
use jsonwebtoken::{
    decode, encode, Algorithm, DecodingKey, EncodingKey, Header, TokenData, Validation,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use nucleus_core::error::{AppError, AuthError};
use nucleus_core::types::*;

/// JWT claims embedded in every token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NucleusClaims {
    // Standard JWT claims
    pub sub: String,          // user_id
    pub iss: String,          // issuer URL
    pub aud: String,          // project_id
    pub exp: i64,             // expiry timestamp
    pub iat: i64,             // issued at
    pub jti: String,          // unique token ID

    // Nucleus-specific claims
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email_verified: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,

    // Organization context
    #[serde(skip_serializing_if = "Option::is_none")]
    pub org_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub org_slug: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub org_role: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub org_permissions: Option<Vec<String>>,
}

/// Signing key pair for JWT
pub struct SigningKeyPair {
    pub kid: String,
    pub private_key_pem: Vec<u8>,
    pub public_key_pem: Vec<u8>,
    pub algorithm: Algorithm,
}

/// JWKS entry for public key exposure
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JwkEntry {
    pub kty: String,
    pub kid: String,
    pub alg: String,
    #[serde(rename = "use")]
    pub use_: String,
    pub n: String, // RSA modulus
    pub e: String, // RSA exponent
}

/// JWKS (JSON Web Key Set)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Jwks {
    pub keys: Vec<JwkEntry>,
}

pub struct JwtService;

impl JwtService {
    /// Generate a new RSA 2048-bit key pair for JWT signing
    pub fn generate_key_pair(kid: &str) -> Result<SigningKeyPair, AppError> {
        use rand::rngs::OsRng;
        use rsa::pkcs8::EncodePrivateKey;
        use rsa::pkcs8::EncodePublicKey;
        use rsa::RsaPrivateKey;

        let mut rng = OsRng;
        let private_key = RsaPrivateKey::new(&mut rng, 2048)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to generate RSA key: {}", e)))?;

        let private_key_pem = private_key
            .to_pkcs8_pem(rsa::pkcs8::LineEnding::LF)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to encode private key: {}", e)))?;

        let public_key_pem = private_key
            .to_public_key()
            .to_public_key_pem(rsa::pkcs8::LineEnding::LF)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to encode public key: {}", e)))?;

        Ok(SigningKeyPair {
            kid: kid.to_string(),
            private_key_pem: private_key_pem.as_bytes().to_vec(),
            public_key_pem: public_key_pem.into_bytes(),
            algorithm: Algorithm::RS256,
        })
    }

    /// Sign a JWT with the given claims and key
    pub fn sign(claims: &NucleusClaims, key: &SigningKeyPair) -> Result<String, AppError> {
        let mut header = Header::new(key.algorithm);
        header.kid = Some(key.kid.clone());

        let encoding_key = EncodingKey::from_rsa_pem(&key.private_key_pem)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Invalid signing key: {}", e)))?;

        encode(&header, claims, &encoding_key)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to sign JWT: {}", e)))
    }

    /// Verify a JWT and return the claims
    /// SECURITY: Only accepts RS256 algorithm (rejects none, HS256, etc.)
    pub fn verify(token: &str, public_key_pem: &[u8]) -> Result<NucleusClaims, AppError> {
        let decoding_key = DecodingKey::from_rsa_pem(public_key_pem)
            .map_err(|_| AppError::Auth(AuthError::TokenInvalid))?;

        let mut validation = Validation::new(Algorithm::RS256);
        validation.validate_exp = true;
        validation.validate_aud = false; // we validate audience manually
        validation.leeway = 0; // strict expiry checking, no grace period
        // SECURITY: algorithms list only contains RS256
        // This prevents algorithm confusion attacks
        validation.algorithms = vec![Algorithm::RS256];

        let token_data: TokenData<NucleusClaims> = decode(token, &decoding_key, &validation)
            .map_err(|e| match e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
                    AppError::Auth(AuthError::TokenExpired)
                }
                _ => AppError::Auth(AuthError::TokenInvalid),
            })?;

        Ok(token_data.claims)
    }

    /// Build claims for a user
    pub fn build_claims(
        user_id: &UserId,
        project_id: &ProjectId,
        issuer: &str,
        lifetime_secs: i64,
        email: Option<String>,
        first_name: Option<String>,
        last_name: Option<String>,
        metadata: Option<serde_json::Value>,
    ) -> NucleusClaims {
        let now = Utc::now();
        NucleusClaims {
            sub: user_id.to_string(),
            iss: issuer.to_string(),
            aud: project_id.to_string(),
            exp: (now + Duration::seconds(lifetime_secs)).timestamp(),
            iat: now.timestamp(),
            jti: format!("jti_{}", Uuid::new_v4()),
            email,
            first_name,
            last_name,
            avatar_url: None,
            email_verified: None,
            metadata,
            org_id: None,
            org_slug: None,
            org_role: None,
            org_permissions: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::Engine;

    fn test_key_pair() -> SigningKeyPair {
        JwtService::generate_key_pair("test-kid-1").unwrap()
    }

    fn test_claims() -> NucleusClaims {
        JwtService::build_claims(
            &UserId::new(),
            &ProjectId::new(),
            "https://nucleus.test",
            300, // 5 minutes
            Some("test@example.com".to_string()),
            Some("John".to_string()),
            Some("Doe".to_string()),
            None,
        )
    }

    #[test]
    fn generate_key_pair_succeeds() {
        let kp = test_key_pair();
        assert_eq!(kp.kid, "test-kid-1");
        assert!(kp.private_key_pem.len() > 100);
        assert!(kp.public_key_pem.len() > 100);
    }

    #[test]
    fn sign_and_verify_roundtrip() {
        let kp = test_key_pair();
        let claims = test_claims();
        let token = JwtService::sign(&claims, &kp).unwrap();
        let verified = JwtService::verify(&token, &kp.public_key_pem).unwrap();
        assert_eq!(verified.sub, claims.sub);
        assert_eq!(verified.email, claims.email);
        assert_eq!(verified.jti, claims.jti);
    }

    #[test]
    fn verify_rejects_expired_token() {
        let kp = test_key_pair();
        let mut claims = test_claims();
        claims.exp = (Utc::now() - Duration::seconds(60)).timestamp(); // expired 1 min ago
        let token = JwtService::sign(&claims, &kp).unwrap();
        let result = JwtService::verify(&token, &kp.public_key_pem);
        assert!(matches!(
            result,
            Err(AppError::Auth(AuthError::TokenExpired))
        ));
    }

    #[test]
    fn verify_rejects_tampered_token() {
        let kp = test_key_pair();
        let claims = test_claims();
        let token = JwtService::sign(&claims, &kp).unwrap();
        // Tamper with payload
        let parts: Vec<&str> = token.split('.').collect();
        let tampered = format!("{}.{}x.{}", parts[0], parts[1], parts[2]);
        let result = JwtService::verify(&tampered, &kp.public_key_pem);
        assert!(matches!(
            result,
            Err(AppError::Auth(AuthError::TokenInvalid))
        ));
    }

    #[test]
    fn verify_rejects_wrong_key() {
        let kp1 = test_key_pair();
        let kp2 = JwtService::generate_key_pair("other-kid").unwrap();
        let claims = test_claims();
        let token = JwtService::sign(&claims, &kp1).unwrap();
        let result = JwtService::verify(&token, &kp2.public_key_pem);
        assert!(matches!(
            result,
            Err(AppError::Auth(AuthError::TokenInvalid))
        ));
    }

    #[test]
    fn claims_include_all_fields() {
        let kp = test_key_pair();
        let mut claims = test_claims();
        claims.org_id = Some("org_123".to_string());
        claims.org_role = Some("admin".to_string());
        claims.org_permissions = Some(vec!["billing:read".to_string()]);
        let token = JwtService::sign(&claims, &kp).unwrap();
        let verified = JwtService::verify(&token, &kp.public_key_pem).unwrap();
        assert_eq!(verified.org_id, Some("org_123".to_string()));
        assert_eq!(verified.org_role, Some("admin".to_string()));
        assert_eq!(
            verified.org_permissions,
            Some(vec!["billing:read".to_string()])
        );
    }

    #[test]
    fn build_claims_generates_unique_jti() {
        let user_id = UserId::new();
        let project_id = ProjectId::new();
        let c1 =
            JwtService::build_claims(&user_id, &project_id, "iss", 300, None, None, None, None);
        let c2 =
            JwtService::build_claims(&user_id, &project_id, "iss", 300, None, None, None, None);
        assert_ne!(c1.jti, c2.jti);
    }

    #[test]
    fn build_claims_sets_correct_expiry() {
        let claims = test_claims();
        let expected_exp = Utc::now().timestamp() + 300;
        // Allow 2 second tolerance
        assert!((claims.exp - expected_exp).abs() < 2);
    }

    #[test]
    fn token_header_includes_kid() {
        let kp = test_key_pair();
        let claims = test_claims();
        let token = JwtService::sign(&claims, &kp).unwrap();
        let header_b64 = token.split('.').next().unwrap();
        let header_bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(header_b64)
            .unwrap();
        let header: serde_json::Value = serde_json::from_slice(&header_bytes).unwrap();
        assert_eq!(header["kid"], "test-kid-1");
        assert_eq!(header["alg"], "RS256");
    }

    #[test]
    fn optional_claims_skipped_when_none() {
        let kp = test_key_pair();
        let claims = JwtService::build_claims(
            &UserId::new(),
            &ProjectId::new(),
            "iss",
            300,
            None,
            None,
            None,
            None,
        );
        let token = JwtService::sign(&claims, &kp).unwrap();
        let payload_b64 = token.split('.').nth(1).unwrap();
        let payload_bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(payload_b64)
            .unwrap();
        let payload: serde_json::Value = serde_json::from_slice(&payload_bytes).unwrap();
        assert!(payload.get("email").is_none());
        assert!(payload.get("org_id").is_none());
    }
}
