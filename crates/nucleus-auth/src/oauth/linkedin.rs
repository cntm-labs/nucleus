use std::sync::Arc;

use async_trait::async_trait;
use nucleus_core::error::{AppError, AuthError};
use serde::Deserialize;

use super::provider::{AuthorizationUrl, HttpClient, OAuthConfig, OAuthProvider, OAuthUserInfo};

const AUTH_URL: &str = "https://www.linkedin.com/oauth/v2/authorization";
const TOKEN_URL: &str = "https://www.linkedin.com/oauth/v2/accessToken";
const USERINFO_URL: &str = "https://api.linkedin.com/v2/userinfo";

pub struct LinkedInProvider {
    config: OAuthConfig,
    http_client: Arc<dyn HttpClient>,
}

impl LinkedInProvider {
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

/// LinkedIn uses OpenID Connect userinfo endpoint.
#[derive(Debug, Deserialize)]
struct LinkedInUserInfo {
    sub: String,
    email: Option<String>,
    name: Option<String>,
    given_name: Option<String>,
    family_name: Option<String>,
    picture: Option<String>,
}

#[async_trait]
impl OAuthProvider for LinkedInProvider {
    fn provider_name(&self) -> &str {
        "linkedin"
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
                "failed to parse LinkedIn token response: {}",
                e
            )))
        })?;

        let user_body = self
            .http_client
            .get_with_bearer(USERINFO_URL, &token_resp.access_token)
            .await?;
        let li_user: LinkedInUserInfo = serde_json::from_str(&user_body).map_err(|e| {
            AppError::Auth(AuthError::OAuthProviderError(format!(
                "failed to parse LinkedIn user info: {}",
                e
            )))
        })?;

        Ok(OAuthUserInfo {
            provider: "linkedin".to_string(),
            provider_user_id: li_user.sub,
            email: li_user.email,
            name: li_user.name,
            first_name: li_user.given_name,
            last_name: li_user.family_name,
            avatar_url: li_user.picture,
            raw_data: serde_json::to_value(&user_body).unwrap_or_default(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> OAuthConfig {
        OAuthConfig {
            client_id: "linkedin-client-id".to_string(),
            client_secret: "linkedin-secret".to_string(),
            redirect_url: "https://app.example.com/callback".to_string(),
            scopes: vec![],
        }
    }

    fn mock_http() -> Arc<dyn HttpClient> {
        Arc::new(crate::oauth::provider::tests::MockHttpClient::new())
    }

    #[test]
    fn linkedin_authorization_url_correct() {
        let provider = LinkedInProvider::new(test_config(), mock_http());
        let result = provider.authorization_url("test-state", None).unwrap();

        assert!(result.url.starts_with(AUTH_URL));
        assert!(result.url.contains("client_id=linkedin-client-id"));
        assert!(result.url.contains("state=test-state"));
        assert!(result.url.contains("scope=openid+profile+email"));
        assert_eq!(result.state, "test-state");
    }

    #[test]
    fn linkedin_authorization_url_with_pkce() {
        let provider = LinkedInProvider::new(test_config(), mock_http());
        let result = provider
            .authorization_url("state123", Some("challenge-value"))
            .unwrap();

        assert!(result.url.contains("code_challenge=challenge-value"));
        assert!(result.url.contains("code_challenge_method=S256"));
    }

    #[tokio::test]
    async fn linkedin_exchange_code_success() {
        let mock = crate::oauth::provider::tests::MockHttpClient::new()
            .with_post_response(TOKEN_URL, r#"{"access_token":"li-token-123"}"#)
            .with_get_response(
                USERINFO_URL,
                r#"{"sub":"urn:li:person:abc123","email":"user@linkedin.com","name":"Test User","given_name":"Test","family_name":"User","picture":"https://media.licdn.com/photo.jpg"}"#,
            );

        let provider = LinkedInProvider::new(test_config(), Arc::new(mock));
        let result = provider.exchange_code("auth-code", None).await.unwrap();

        assert_eq!(result.provider, "linkedin");
        assert_eq!(result.provider_user_id, "urn:li:person:abc123");
        assert_eq!(result.email, Some("user@linkedin.com".to_string()));
        assert_eq!(result.name, Some("Test User".to_string()));
        assert_eq!(result.first_name, Some("Test".to_string()));
        assert_eq!(result.last_name, Some("User".to_string()));
        assert_eq!(
            result.avatar_url,
            Some("https://media.licdn.com/photo.jpg".to_string())
        );
    }
}
