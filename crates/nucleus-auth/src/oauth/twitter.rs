use std::sync::Arc;

use async_trait::async_trait;
use nucleus_core::error::{AppError, AuthError};
use serde::Deserialize;

use super::provider::{AuthorizationUrl, HttpClient, OAuthConfig, OAuthProvider, OAuthUserInfo};

const AUTH_URL: &str = "https://twitter.com/i/oauth2/authorize";
const TOKEN_URL: &str = "https://api.twitter.com/2/oauth2/token";
const USERINFO_URL: &str = "https://api.twitter.com/2/users/me?user.fields=profile_image_url,name";

pub struct TwitterProvider {
    config: OAuthConfig,
    http_client: Arc<dyn HttpClient>,
}

impl TwitterProvider {
    pub fn new(config: OAuthConfig, http_client: Arc<dyn HttpClient>) -> Self {
        Self {
            config,
            http_client,
        }
    }

    pub fn default_scopes() -> Vec<String> {
        vec![
            "users.read".to_string(),
            "tweet.read".to_string(),
            "offline.access".to_string(),
        ]
    }
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
}

/// Twitter wraps user data in a `data` envelope.
#[derive(Debug, Deserialize)]
struct TwitterResponse {
    data: TwitterUserInfo,
}

#[derive(Debug, Deserialize)]
struct TwitterUserInfo {
    id: String,
    name: Option<String>,
    profile_image_url: Option<String>,
}

#[async_trait]
impl OAuthProvider for TwitterProvider {
    fn provider_name(&self) -> &str {
        "twitter"
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

        // Twitter OAuth 2.0 requires PKCE
        let challenge = pkce_challenge.ok_or_else(|| {
            AppError::Auth(AuthError::OAuthProviderError(
                "Twitter OAuth 2.0 requires PKCE".to_string(),
            ))
        })?;

        let mut url = url::Url::parse(AUTH_URL)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("invalid auth URL: {}", e)))?;

        {
            let mut params = url.query_pairs_mut();
            params.append_pair("response_type", "code");
            params.append_pair("client_id", &self.config.client_id);
            params.append_pair("redirect_uri", &self.config.redirect_url);
            params.append_pair("scope", &scopes);
            params.append_pair("state", state);
            params.append_pair("code_challenge", challenge);
            params.append_pair("code_challenge_method", "S256");
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
        let verifier = pkce_verifier.ok_or_else(|| {
            AppError::Auth(AuthError::OAuthProviderError(
                "Twitter OAuth 2.0 requires PKCE verifier".to_string(),
            ))
        })?;

        // Twitter uses Basic auth (client_id:client_secret) for token exchange
        let params = vec![
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", self.config.redirect_url.as_str()),
            ("code_verifier", verifier),
        ];

        let token_body = self.http_client.post_form(TOKEN_URL, &params).await?;
        let token_resp: TokenResponse = serde_json::from_str(&token_body).map_err(|e| {
            AppError::Auth(AuthError::OAuthProviderError(format!(
                "failed to parse Twitter token response: {}",
                e
            )))
        })?;

        let user_body = self
            .http_client
            .get_with_bearer(USERINFO_URL, &token_resp.access_token)
            .await?;
        let twitter_resp: TwitterResponse = serde_json::from_str(&user_body).map_err(|e| {
            AppError::Auth(AuthError::OAuthProviderError(format!(
                "failed to parse Twitter user info: {}",
                e
            )))
        })?;

        let user = twitter_resp.data;

        Ok(OAuthUserInfo {
            provider: "twitter".to_string(),
            provider_user_id: user.id,
            email: None, // Twitter OAuth 2.0 does not provide email
            name: user.name.clone(),
            first_name: user.name,
            last_name: None,
            avatar_url: user.profile_image_url,
            raw_data: serde_json::to_value(&user_body).unwrap_or_default(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> OAuthConfig {
        OAuthConfig {
            client_id: "twitter-client-id".to_string(),
            client_secret: "twitter-secret".to_string(),
            redirect_url: "https://app.example.com/callback".to_string(),
            scopes: vec![],
        }
    }

    fn mock_http() -> Arc<dyn HttpClient> {
        Arc::new(crate::oauth::provider::tests::MockHttpClient::new())
    }

    #[test]
    fn twitter_requires_pkce() {
        let provider = TwitterProvider::new(test_config(), mock_http());
        let result = provider.authorization_url("test-state", None);
        assert!(result.is_err());
    }

    #[test]
    fn twitter_authorization_url_with_pkce() {
        let provider = TwitterProvider::new(test_config(), mock_http());
        let result = provider
            .authorization_url("test-state", Some("challenge-value"))
            .unwrap();

        assert!(result.url.starts_with(AUTH_URL));
        assert!(result.url.contains("client_id=twitter-client-id"));
        assert!(result.url.contains("state=test-state"));
        assert!(result.url.contains("code_challenge=challenge-value"));
        assert!(result.url.contains("code_challenge_method=S256"));
        assert!(result.url.contains("scope=users.read"));
    }

    #[tokio::test]
    async fn twitter_exchange_code_requires_pkce_verifier() {
        let provider = TwitterProvider::new(test_config(), mock_http());
        let result = provider.exchange_code("auth-code", None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn twitter_exchange_code_success() {
        let mock = crate::oauth::provider::tests::MockHttpClient::new()
            .with_post_response(TOKEN_URL, r#"{"access_token":"twitter-token-123"}"#)
            .with_get_response(
                USERINFO_URL,
                r#"{"data":{"id":"11111","name":"Test User","username":"testuser","profile_image_url":"https://pbs.twimg.com/photo.jpg"}}"#,
            );

        let provider = TwitterProvider::new(test_config(), Arc::new(mock));
        let result = provider
            .exchange_code("auth-code", Some("verifier"))
            .await
            .unwrap();

        assert_eq!(result.provider, "twitter");
        assert_eq!(result.provider_user_id, "11111");
        assert_eq!(result.email, None);
        assert_eq!(result.name, Some("Test User".to_string()));
        assert_eq!(
            result.avatar_url,
            Some("https://pbs.twimg.com/photo.jpg".to_string())
        );
    }
}
