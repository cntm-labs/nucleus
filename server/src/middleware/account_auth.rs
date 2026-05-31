//! Account auth middleware — extracts and validates dashboard-account JWT.
//!
//! Differs from user-auth (`middleware/auth.rs`): validates `aud == "nucleus.dashboard"`
//! and asserts `kind == Some("account")`. Returns `TokenInvalid` on signature/validation
//! failure or if presented a user JWT (kind != "account") — defense in depth against
//! token-type confusion.

use std::sync::Arc;

use axum::extract::FromRequestParts;
use axum::http::request::Parts;

use crate::auth::jwt::{JwtService, SigningKeyPair};
use crate::core::error::{AppError, AuthError};
use crate::core::types::AccountId;

const ACCOUNT_AUDIENCE: &str = "nucleus.dashboard";

/// Extension attached to authenticated dashboard requests.
#[derive(Debug, Clone)]
pub struct AuthAccount {
    pub account_id: AccountId,
    pub email: Option<String>,
    pub jti: String,
}

impl<S> FromRequestParts<S> for AuthAccount
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // 1. Extract Authorization: Bearer <jwt>
        let token = parts
            .headers
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.strip_prefix("Bearer "))
            .ok_or(AppError::Auth(AuthError::TokenInvalid))?;

        // 2. Get the signing key from request extensions (router inserts it)
        let signing_key = parts
            .extensions
            .get::<Arc<SigningKeyPair>>()
            .ok_or(AppError::Auth(AuthError::TokenInvalid))?;

        // 3. Verify JWT (ES256, aud=nucleus.dashboard)
        let claims = JwtService::verify(token, &signing_key.public_key_pem, ACCOUNT_AUDIENCE)?;

        // 4. Defense in depth: kind MUST be "account". A user JWT that somehow
        //    lands with aud=nucleus.dashboard would still be rejected here.
        if claims.kind.as_deref() != Some("account") {
            return Err(AppError::Auth(AuthError::TokenInvalid));
        }

        // 5. Parse account_id from sub
        let account_id = claims
            .sub
            .parse::<uuid::Uuid>()
            .map(AccountId::from_uuid)
            .map_err(|_| AppError::Auth(AuthError::TokenInvalid))?;

        Ok(AuthAccount {
            account_id,
            email: claims.email.clone(),
            jti: claims.jti.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::jwt::{JwtService, NucleusClaims};
    use crate::core::types::{ProjectId, SessionId, UserId};
    use axum::http::Request;

    fn signing_key() -> Arc<SigningKeyPair> {
        Arc::new(JwtService::generate_key_pair("test-kid").unwrap())
    }

    fn build_account_jwt(key: &SigningKeyPair, account_id: AccountId) -> String {
        let claims = JwtService::build_account_claims(
            &account_id,
            &SessionId::new(),
            "https://nucleus.test",
            300,
            Some("test@example.com".to_string()),
        );
        JwtService::sign(&claims, key).unwrap()
    }

    fn build_user_jwt(key: &SigningKeyPair) -> String {
        let claims = JwtService::build_claims(
            &UserId::new(),
            &ProjectId::new(),
            &SessionId::new(),
            "https://nucleus.test",
            300,
            Some("user@example.com".to_string()),
            None,
            None,
            None,
        );
        JwtService::sign(&claims, key).unwrap()
    }

    fn build_parts_with(token: Option<&str>, key: Option<Arc<SigningKeyPair>>) -> Parts {
        let mut req_builder = Request::builder().uri("/");
        if let Some(t) = token {
            req_builder = req_builder.header("authorization", format!("Bearer {t}"));
        }
        let mut req = req_builder.body(()).unwrap();
        if let Some(k) = key {
            req.extensions_mut().insert(k);
        }
        req.into_parts().0
    }

    #[tokio::test]
    async fn accepts_valid_account_jwt() {
        let key = signing_key();
        let account_id = AccountId::new();
        let jwt = build_account_jwt(&key, account_id);
        let mut parts = build_parts_with(Some(&jwt), Some(key));

        let auth = AuthAccount::from_request_parts(&mut parts, &())
            .await
            .expect("valid account JWT should authenticate");
        assert_eq!(auth.account_id, account_id);
        assert_eq!(auth.email.as_deref(), Some("test@example.com"));
    }

    #[tokio::test]
    async fn rejects_user_jwt() {
        // User JWT has aud = project_id, NOT "nucleus.dashboard" → JwtService::verify
        // rejects on audience mismatch BEFORE we even check `kind`. That's fine —
        // both layers refuse a user token from the dashboard surface.
        let key = signing_key();
        let jwt = build_user_jwt(&key);
        let mut parts = build_parts_with(Some(&jwt), Some(key));

        let err = AuthAccount::from_request_parts(&mut parts, &())
            .await
            .expect_err("user JWT must not authenticate as account");
        assert!(matches!(err, AppError::Auth(AuthError::TokenInvalid)));
    }

    #[tokio::test]
    async fn rejects_missing_authorization_header() {
        let key = signing_key();
        let mut parts = build_parts_with(None, Some(key));

        let err = AuthAccount::from_request_parts(&mut parts, &())
            .await
            .expect_err("missing token must reject");
        assert!(matches!(err, AppError::Auth(AuthError::TokenInvalid)));
    }

    #[tokio::test]
    async fn rejects_jwt_signed_by_different_key() {
        let key_a = signing_key();
        let key_b = signing_key(); // independent key pair
        let jwt = build_account_jwt(&key_a, AccountId::new());
        // Present token signed by key_a, but middleware has key_b
        let mut parts = build_parts_with(Some(&jwt), Some(key_b));

        let err = AuthAccount::from_request_parts(&mut parts, &())
            .await
            .expect_err("token signed by different key must reject");
        assert!(matches!(err, AppError::Auth(AuthError::TokenInvalid)));
    }

    /// Defense-in-depth: even if a token is somehow signed correctly with the
    /// account audience but `kind` is missing or set to "user", reject it.
    #[tokio::test]
    async fn rejects_account_audience_token_with_wrong_kind() {
        let key = signing_key();
        // Manually craft a claims with aud=nucleus.dashboard but kind=Some("user")
        let claims = NucleusClaims {
            sub: AccountId::new().to_string(),
            iss: "https://nucleus.test".to_string(),
            aud: ACCOUNT_AUDIENCE.to_string(),
            exp: (chrono::Utc::now() + chrono::Duration::seconds(300)).timestamp(),
            iat: chrono::Utc::now().timestamp(),
            jti: format!("jti_{}", uuid::Uuid::new_v4()),
            sid: SessionId::new().to_string(),
            kind: Some("user".to_string()),
            email: None,
            first_name: None,
            last_name: None,
            avatar_url: None,
            email_verified: None,
            metadata: None,
            org_id: None,
            org_slug: None,
            org_role: None,
            org_permissions: None,
        };
        let jwt = JwtService::sign(&claims, &key).unwrap();
        let mut parts = build_parts_with(Some(&jwt), Some(key));

        let err = AuthAccount::from_request_parts(&mut parts, &())
            .await
            .expect_err("kind mismatch must reject even with valid signature + aud");
        assert!(matches!(err, AppError::Auth(AuthError::TokenInvalid)));
    }
}
