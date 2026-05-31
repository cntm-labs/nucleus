use chrono::{Duration, Utc};
use jsonwebtoken::{
    decode, encode, Algorithm, DecodingKey, EncodingKey, Header, TokenData, Validation,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::core::error::{AppError, AuthError};
use crate::core::types::*;

/// JWT claims embedded in every token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NucleusClaims {
    // Standard JWT claims
    pub sub: String, // user_id
    pub iss: String, // issuer URL
    pub aud: String, // project_id
    pub exp: i64,    // expiry timestamp
    pub iat: i64,    // issued at
    pub jti: String, // unique token ID
    pub sid: String, // session_id

    // Token kind discriminator. None or Some("user") = user auth; Some("account") = dashboard account.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,

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

/// JWKS entry for public key exposure (EC key format)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JwkEntry {
    pub kty: String,
    pub kid: String,
    pub alg: String,
    #[serde(rename = "use")]
    pub use_: String,
    pub crv: String, // EC curve name
    pub x: String,   // EC x coordinate (base64url)
    pub y: String,   // EC y coordinate (base64url)
}

/// JWKS (JSON Web Key Set)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Jwks {
    pub keys: Vec<JwkEntry>,
}

impl SigningKeyPair {
    pub fn to_jwk_entry(&self) -> Result<JwkEntry, AppError> {
        use base64::Engine;
        use p256::elliptic_curve::sec1::ToEncodedPoint;
        use p256::pkcs8::DecodePublicKey;

        let public_key = p256::PublicKey::from_public_key_pem(
            std::str::from_utf8(&self.public_key_pem)
                .map_err(|e| AppError::Internal(anyhow::anyhow!("Invalid PEM encoding: {}", e)))?,
        )
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to parse public key: {}", e)))?;

        let point = public_key.to_encoded_point(false);
        let x_bytes = point
            .x()
            .ok_or_else(|| AppError::Internal(anyhow::anyhow!("Missing EC x coordinate")))?;
        let y_bytes = point
            .y()
            .ok_or_else(|| AppError::Internal(anyhow::anyhow!("Missing EC y coordinate")))?;

        let x = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(x_bytes);
        let y = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(y_bytes);

        Ok(JwkEntry {
            kty: "EC".to_string(),
            kid: self.kid.clone(),
            alg: "ES256".to_string(),
            use_: "sig".to_string(),
            crv: "P-256".to_string(),
            x,
            y,
        })
    }

    pub fn to_jwks(&self) -> Result<Jwks, AppError> {
        Ok(Jwks {
            keys: vec![self.to_jwk_entry()?],
        })
    }
}

pub struct JwtService;

impl JwtService {
    pub fn generate_key_pair(kid: &str) -> Result<SigningKeyPair, AppError> {
        use p256::ecdsa::SigningKey;
        use p256::pkcs8::{EncodePrivateKey, EncodePublicKey};

        let signing_key = SigningKey::random(&mut p256::elliptic_curve::rand_core::OsRng);

        let private_key_pem = signing_key
            .to_pkcs8_pem(p256::pkcs8::LineEnding::LF)
            .map_err(|e| {
                AppError::Internal(anyhow::anyhow!("Failed to encode private key: {}", e))
            })?;

        let public_key_pem = signing_key
            .verifying_key()
            .to_public_key_pem(p256::pkcs8::LineEnding::LF)
            .map_err(|e| {
                AppError::Internal(anyhow::anyhow!("Failed to encode public key: {}", e))
            })?;

        Ok(SigningKeyPair {
            kid: kid.to_string(),
            private_key_pem: private_key_pem.as_bytes().to_vec(),
            public_key_pem: public_key_pem.into_bytes(),
            algorithm: Algorithm::ES256,
        })
    }

    pub fn sign(claims: &NucleusClaims, key: &SigningKeyPair) -> Result<String, AppError> {
        let mut header = Header::new(key.algorithm);
        header.kid = Some(key.kid.clone());

        let encoding_key = EncodingKey::from_ec_pem(&key.private_key_pem)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Invalid signing key: {}", e)))?;

        encode(&header, claims, &encoding_key)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to sign JWT: {}", e)))
    }

    pub fn verify(
        token: &str,
        public_key_pem: &[u8],
        expected_audience: &str,
    ) -> Result<NucleusClaims, AppError> {
        let decoding_key = DecodingKey::from_ec_pem(public_key_pem)
            .map_err(|_| AppError::Auth(AuthError::TokenInvalid))?;

        let mut validation = Validation::new(Algorithm::ES256);
        validation.validate_exp = true;
        validation.validate_aud = true;
        validation.set_audience(&[expected_audience]);
        validation.leeway = 0;
        validation.algorithms = vec![Algorithm::ES256];

        let token_data: TokenData<NucleusClaims> = decode(token, &decoding_key, &validation)
            .map_err(|e| match e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
                    AppError::Auth(AuthError::TokenExpired)
                }
                _ => AppError::Auth(AuthError::TokenInvalid),
            })?;

        Ok(token_data.claims)
    }

    pub fn build_claims(
        user_id: &UserId,
        project_id: &ProjectId,
        session_id: &SessionId,
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
            sid: session_id.to_string(),
            kind: None,
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

    pub fn build_account_claims(
        account_id: &AccountId,
        session_id: &SessionId,
        issuer: &str,
        lifetime_secs: i64,
        email: Option<String>,
    ) -> NucleusClaims {
        let now = Utc::now();
        NucleusClaims {
            sub: account_id.to_string(),
            iss: issuer.to_string(),
            aud: "nucleus.dashboard".to_string(),
            exp: (now + Duration::seconds(lifetime_secs)).timestamp(),
            iat: now.timestamp(),
            jti: format!("jti_{}", Uuid::new_v4()),
            sid: session_id.to_string(),
            kind: Some("account".to_string()),
            email,
            first_name: None,
            last_name: None,
            avatar_url: None,
            email_verified: None,
            metadata: None,
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

    fn test_key_pair() -> SigningKeyPair {
        JwtService::generate_key_pair("test-kid-1").unwrap()
    }

    fn test_claims() -> NucleusClaims {
        JwtService::build_claims(
            &UserId::new(),
            &ProjectId::new(),
            &SessionId::new(),
            "https://nucleus.test",
            300,
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
    }

    #[test]
    fn sign_and_verify_roundtrip() {
        let kp = test_key_pair();
        let claims = test_claims();
        let token = JwtService::sign(&claims, &kp).unwrap();
        let verified = JwtService::verify(&token, &kp.public_key_pem, &claims.aud).unwrap();
        assert_eq!(verified.sub, claims.sub);
        assert_eq!(verified.email, claims.email);
        assert_eq!(verified.jti, claims.jti);
        assert_eq!(verified.sid, claims.sid);
    }
}
