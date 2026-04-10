use std::sync::Arc;

use crate::core::error::{AppError, AuthError};
use async_trait::async_trait;
use serde::Deserialize;

use super::provider::{AuthorizationUrl, HttpClient, OAuthConfig, OAuthProvider, OAuthUserInfo};

const AUTH_URL: &str = "https://www.facebook.com/v19.0/dialog/oauth";
const TOKEN_URL: &str = "https://graph.facebook.com/v19.0/oauth/access_token";
const USERINFO_URL: &str =
    "https://graph.facebook.com/v19.0/me?fields=id,email,name,first_name,last_name,picture.type(large)";

pub struct FacebookProvider {
    config: OAuthConfig,
    http_client: Arc<dyn HttpClient>,
}

impl FacebookProvider {
    pub fn new(config: OAuthConfig, http_client: Arc<dyn HttpClient>) -> Self {
        Self {
            config,
            http_client,
        }
    }

    pub fn default_scopes() -> Vec<String> {
        vec!["email".to_string(), "public_profile".to_string()]
    }
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
}

#[derive(Debug, Deserialize)]
struct FacebookUserInfo {
    id: String,
    email: Option<String>,
    name: Option<String>,
    first_name: Option<String>,
    last_name: Option<String>,
    picture: Option<FacebookPicture>,
}

#[derive(Debug, Deserialize)]
struct FacebookPicture {
    data: Option<FacebookPictureData>,
}

#[derive(Debug, Deserialize)]
struct FacebookPictureData {
    url: Option<String>,
}

#[async_trait]
impl OAuthProvider for FacebookProvider {
    fn provider_name(&self) -> &str {
        "facebook"
    }

    fn authorization_url(
        &self,
        state: &str,
        pkce_challenge: Option<&str>,
    ) -> Result<AuthorizationUrl, AppError> {
        let scopes = if self.config.scopes.is_empty() {
            Self::default_scopes().join(",")
        } else {
            self.config.scopes.join(",")
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
            ("client_id", self.config.client_id.as_str()),
            ("client_secret", self.config.client_secret.as_str()),
            ("redirect_uri", self.config.redirect_url.as_str()),
            ("code", code),
        ];

        let verifier_owned;
        if let Some(v) = pkce_verifier {
            verifier_owned = v.to_string();
            params.push(("code_verifier", verifier_owned.as_str()));
        }

        let token_body = self.http_client.post_form(TOKEN_URL, &params).await?;
        let token_resp: TokenResponse = serde_json::from_str(&token_body).map_err(|e| {
            AppError::Auth(AuthError::OAuthProviderError(format!(
                "failed to parse Facebook token response: {}",
                e
            )))
        })?;

        let user_body = self
            .http_client
            .get_with_bearer(USERINFO_URL, &token_resp.access_token)
            .await?;
        let fb_user: FacebookUserInfo = serde_json::from_str(&user_body).map_err(|e| {
            AppError::Auth(AuthError::OAuthProviderError(format!(
                "failed to parse Facebook user info: {}",
                e
            )))
        })?;

        Ok(OAuthUserInfo {
            provider: "facebook".to_string(),
            provider_user_id: fb_user.id,
            email: fb_user.email,
            name: fb_user.name,
            first_name: fb_user.first_name,
            last_name: fb_user.last_name,
            avatar_url: fb_user.picture.and_then(|p| p.data).and_then(|d| d.url),
            raw_data: serde_json::to_value(&user_body).unwrap_or_default(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> OAuthConfig {
        OAuthConfig {
            client_id: "fb-client-id".to_string(),
            client_secret: "fb-secret".to_string(),
            redirect_url: "https://app.example.com/callback".to_string(),
            scopes: vec![],
        }
    }

    fn mock_http() -> Arc<dyn HttpClient> {
        Arc::new(crate::auth::oauth::provider::tests::MockHttpClient::new())
    }

    #[test]
    fn facebook_authorization_url_correct() {
        let provider = FacebookProvider::new(test_config(), mock_http());
        let result = provider.authorization_url("test-state", None).unwrap();

        assert!(result.url.starts_with(AUTH_URL));
        assert!(result.url.contains("client_id=fb-client-id"));
        assert!(result.url.contains("state=test-state"));
        assert!(result.url.contains("response_type=code"));
        assert!(result.url.contains("scope=email"));
        assert!(result.url.contains("redirect_uri="));
        assert_eq!(result.state, "test-state");
    }

    #[test]
    fn facebook_authorization_url_with_pkce() {
        let provider = FacebookProvider::new(test_config(), mock_http());
        let result = provider
            .authorization_url("state123", Some("challenge-value"))
            .unwrap();

        assert!(result.url.contains("code_challenge=challenge-value"));
        assert!(result.url.contains("code_challenge_method=S256"));
    }

    #[tokio::test]
    async fn facebook_exchange_code_success() {
        let mock = crate::auth::oauth::provider::tests::MockHttpClient::new()
            .with_post_response(TOKEN_URL, r#"{"access_token":"fb-token-123"}"#)
            .with_get_response(
                USERINFO_URL,
                r#"{"id":"12345","email":"user@fb.com","name":"Test User","first_name":"Test","last_name":"User","picture":{"data":{"url":"https://fb.com/photo.jpg"}}}"#,
            );

        let provider = FacebookProvider::new(test_config(), Arc::new(mock));
        let result = provider.exchange_code("auth-code", None).await.unwrap();

        assert_eq!(result.provider, "facebook");
        assert_eq!(result.provider_user_id, "12345");
        assert_eq!(result.email, Some("user@fb.com".to_string()));
        assert_eq!(result.name, Some("Test User".to_string()));
        assert_eq!(result.first_name, Some("Test".to_string()));
        assert_eq!(result.last_name, Some("User".to_string()));
        assert_eq!(
            result.avatar_url,
            Some("https://fb.com/photo.jpg".to_string())
        );
    }
}
