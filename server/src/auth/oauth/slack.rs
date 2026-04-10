use std::sync::Arc;

use crate::core::error::{AppError, AuthError};
use async_trait::async_trait;
use serde::Deserialize;

use super::provider::{AuthorizationUrl, HttpClient, OAuthConfig, OAuthProvider, OAuthUserInfo};

const AUTH_URL: &str = "https://slack.com/openid/connect/authorize";
const TOKEN_URL: &str = "https://slack.com/api/openid.connect.token";
const USERINFO_URL: &str = "https://slack.com/api/openid.connect.userInfo";

pub struct SlackProvider {
    config: OAuthConfig,
    http_client: Arc<dyn HttpClient>,
}

impl SlackProvider {
    pub fn new(config: OAuthConfig, http_client: Arc<dyn HttpClient>) -> Self {
        Self {
            config,
            http_client,
        }
    }

    pub fn default_scopes() -> Vec<String> {
        vec![
            "openid".to_string(),
            "profile".to_string(),
            "email".to_string(),
        ]
    }
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
}

/// Slack OpenID Connect userinfo response.
#[derive(Debug, Deserialize)]
struct SlackUserInfo {
    sub: String,
    email: Option<String>,
    name: Option<String>,
    given_name: Option<String>,
    family_name: Option<String>,
    picture: Option<String>,
}

#[async_trait]
impl OAuthProvider for SlackProvider {
    fn provider_name(&self) -> &str {
        "slack"
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
                "failed to parse Slack token response: {}",
                e
            )))
        })?;

        let user_body = self
            .http_client
            .get_with_bearer(USERINFO_URL, &token_resp.access_token)
            .await?;
        let slack_user: SlackUserInfo = serde_json::from_str(&user_body).map_err(|e| {
            AppError::Auth(AuthError::OAuthProviderError(format!(
                "failed to parse Slack user info: {}",
                e
            )))
        })?;

        Ok(OAuthUserInfo {
            provider: "slack".to_string(),
            provider_user_id: slack_user.sub,
            email: slack_user.email,
            name: slack_user.name,
            first_name: slack_user.given_name,
            last_name: slack_user.family_name,
            avatar_url: slack_user.picture,
            raw_data: serde_json::to_value(&user_body).unwrap_or_default(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> OAuthConfig {
        OAuthConfig {
            client_id: "slack-client-id".to_string(),
            client_secret: "slack-secret".to_string(),
            redirect_url: "https://app.example.com/callback".to_string(),
            scopes: vec![],
        }
    }

    fn mock_http() -> Arc<dyn HttpClient> {
        Arc::new(crate::auth::oauth::provider::tests::MockHttpClient::new())
    }

    #[test]
    fn slack_authorization_url_correct() {
        let provider = SlackProvider::new(test_config(), mock_http());
        let result = provider.authorization_url("test-state", None).unwrap();

        assert!(result.url.starts_with(AUTH_URL));
        assert!(result.url.contains("client_id=slack-client-id"));
        assert!(result.url.contains("state=test-state"));
        assert!(result.url.contains("scope=openid+profile+email"));
        assert_eq!(result.state, "test-state");
    }

    #[test]
    fn slack_authorization_url_with_pkce() {
        let provider = SlackProvider::new(test_config(), mock_http());
        let result = provider
            .authorization_url("state123", Some("challenge-value"))
            .unwrap();

        assert!(result.url.contains("code_challenge=challenge-value"));
        assert!(result.url.contains("code_challenge_method=S256"));
    }

    #[tokio::test]
    async fn slack_exchange_code_success() {
        let mock = crate::auth::oauth::provider::tests::MockHttpClient::new()
            .with_post_response(TOKEN_URL, r#"{"access_token":"slack-token-123"}"#)
            .with_get_response(
                USERINFO_URL,
                r#"{"sub":"U12345","email":"user@slack.com","name":"Test User","given_name":"Test","family_name":"User","picture":"https://avatars.slack-edge.com/photo.jpg"}"#,
            );

        let provider = SlackProvider::new(test_config(), Arc::new(mock));
        let result = provider.exchange_code("auth-code", None).await.unwrap();

        assert_eq!(result.provider, "slack");
        assert_eq!(result.provider_user_id, "U12345");
        assert_eq!(result.email, Some("user@slack.com".to_string()));
        assert_eq!(result.name, Some("Test User".to_string()));
        assert_eq!(result.first_name, Some("Test".to_string()));
        assert_eq!(result.last_name, Some("User".to_string()));
        assert_eq!(
            result.avatar_url,
            Some("https://avatars.slack-edge.com/photo.jpg".to_string())
        );
    }
}
