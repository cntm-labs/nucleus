use std::sync::Arc;

use crate::core::error::{AppError, AuthError};
use async_trait::async_trait;
use serde::Deserialize;

use super::provider::{AuthorizationUrl, HttpClient, OAuthConfig, OAuthProvider, OAuthUserInfo};

const AUTH_URL: &str = "https://appleid.apple.com/auth/authorize";
const TOKEN_URL: &str = "https://appleid.apple.com/auth/token";

pub struct AppleProvider {
    config: OAuthConfig,
    http_client: Arc<dyn HttpClient>,
}

impl AppleProvider {
    pub fn new(config: OAuthConfig, http_client: Arc<dyn HttpClient>) -> Self {
        Self {
            config,
            http_client,
        }
    }

    /// Return default scopes for Apple Sign In.
    pub fn default_scopes() -> Vec<String> {
        vec!["name".to_string(), "email".to_string()]
    }

    /// Decode the JWT ID token payload (base64url) without signature verification.
    /// In production, you would verify the signature against Apple's public keys.
    /// Returns the decoded claims as a JSON value.
    fn decode_id_token_payload(id_token: &str) -> Result<AppleIdTokenClaims, AppError> {
        let parts: Vec<&str> = id_token.split('.').collect();
        if parts.len() != 3 {
            return Err(AppError::Auth(AuthError::OAuthProviderError(
                "invalid Apple ID token format".to_string(),
            )));
        }

        let payload_bytes =
            base64::Engine::decode(&base64::engine::general_purpose::URL_SAFE_NO_PAD, parts[1])
                .map_err(|e| {
                    AppError::Auth(AuthError::OAuthProviderError(format!(
                        "failed to decode Apple ID token payload: {}",
                        e
                    )))
                })?;

        serde_json::from_slice(&payload_bytes).map_err(|e| {
            AppError::Auth(AuthError::OAuthProviderError(format!(
                "failed to parse Apple ID token claims: {}",
                e
            )))
        })
    }
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    id_token: String,
}

#[derive(Debug, Deserialize)]
struct AppleIdTokenClaims {
    sub: String,
    email: Option<String>,
}

#[async_trait]
impl OAuthProvider for AppleProvider {
    fn provider_name(&self) -> &str {
        "apple"
    }

    fn authorization_url(
        &self,
        state: &str,
        pkce_challenge: Option<&str>,
    ) -> Result<AuthorizationUrl, AppError> {
        let scopes = if self.config.scopes.is_empty() {
            Self::default_scopes().join(" ")
        } else {
            self.config.scopes.join(" ")
        };

        let mut url = url::Url::parse(AUTH_URL)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("invalid auth URL: {}", e)))?;

        {
            let mut params = url.query_pairs_mut();
            params.append_pair("response_type", "code");
            params.append_pair("response_mode", "form_post");
            params.append_pair("client_id", &self.config.client_id);
            params.append_pair("redirect_uri", &self.config.redirect_url);
            params.append_pair("scope", &scopes);
            params.append_pair("state", state);

            if let Some(challenge) = pkce_challenge {
                params.append_pair("code_challenge", challenge);
                params.append_pair("code_challenge_method", "S256");
            }
        }

        Ok(AuthorizationUrl {
            url: url.to_string(),
            state: state.to_string(),
            pkce_verifier: None,
        })
    }

    async fn exchange_code(
        &self,
        code: &str,
        pkce_verifier: Option<&str>,
    ) -> Result<OAuthUserInfo, AppError> {
        let mut params = vec![
            ("grant_type", "authorization_code"),
            ("code", code),
            ("client_id", self.config.client_id.as_str()),
            ("client_secret", self.config.client_secret.as_str()),
            ("redirect_uri", self.config.redirect_url.as_str()),
        ];

        let verifier_owned;
        if let Some(v) = pkce_verifier {
            verifier_owned = v.to_string();
            params.push(("code_verifier", verifier_owned.as_str()));
        }

        let token_body = self.http_client.post_form(TOKEN_URL, &params).await?;
        let token_resp: TokenResponse = serde_json::from_str(&token_body).map_err(|e| {
            AppError::Auth(AuthError::OAuthProviderError(format!(
                "failed to parse Apple token response: {}",
                e
            )))
        })?;

        // Apple uses the ID token (JWT) for user info instead of a userinfo endpoint
        let claims = Self::decode_id_token_payload(&token_resp.id_token)?;

        Ok(OAuthUserInfo {
            provider: "apple".to_string(),
            provider_user_id: claims.sub,
            email: claims.email,
            name: None,
            first_name: None,
            last_name: None,
            avatar_url: None,
            raw_data: serde_json::to_value(&token_body).unwrap_or_default(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> OAuthConfig {
        OAuthConfig {
            client_id: "apple-client-id".to_string(),
            client_secret: "apple-secret".to_string(),
            redirect_url: "https://app.example.com/callback".to_string(),
            scopes: vec![],
        }
    }

    fn mock_http() -> Arc<dyn HttpClient> {
        Arc::new(crate::auth::oauth::provider::tests::MockHttpClient::new())
    }

    #[test]
    fn apple_authorization_url_correct() {
        let provider = AppleProvider::new(test_config(), mock_http());
        let result = provider.authorization_url("test-state", None).unwrap();

        assert!(result.url.starts_with(AUTH_URL));
        assert!(result.url.contains("client_id=apple-client-id"));
        assert!(result.url.contains("state=test-state"));
        assert!(result.url.contains("response_type=code"));
        assert!(result.url.contains("response_mode=form_post"));
        assert!(result.url.contains("scope=name+email"));
        assert_eq!(result.state, "test-state");
    }

    #[test]
    fn apple_decode_id_token_payload() {
        // Build a minimal JWT: header.payload.signature
        let payload = serde_json::json!({
            "sub": "apple-user-123",
            "email": "user@privaterelay.appleid.com"
        });
        let payload_b64 = base64::Engine::encode(
            &base64::engine::general_purpose::URL_SAFE_NO_PAD,
            serde_json::to_vec(&payload).unwrap(),
        );
        let fake_jwt = format!("eyJhbGciOiJSUzI1NiJ9.{}.fake-signature", payload_b64);

        let claims = AppleProvider::decode_id_token_payload(&fake_jwt).unwrap();
        assert_eq!(claims.sub, "apple-user-123");
        assert_eq!(
            claims.email,
            Some("user@privaterelay.appleid.com".to_string())
        );
    }
}
