use std::sync::Arc;

use async_trait::async_trait;
use nucleus_core::error::{AppError, AuthError};
use serde::Deserialize;

use super::provider::{AuthorizationUrl, HttpClient, OAuthConfig, OAuthProvider, OAuthUserInfo};

const AUTH_URL: &str = "https://github.com/login/oauth/authorize";
const TOKEN_URL: &str = "https://github.com/login/oauth/access_token";
const USERINFO_URL: &str = "https://api.github.com/user";

pub struct GitHubProvider {
    config: OAuthConfig,
    http_client: Arc<dyn HttpClient>,
}

impl GitHubProvider {
    pub fn new(config: OAuthConfig, http_client: Arc<dyn HttpClient>) -> Self {
        Self {
            config,
            http_client,
        }
    }

    /// Return default scopes for GitHub OAuth.
    pub fn default_scopes() -> Vec<String> {
        vec!["read:user".to_string(), "user:email".to_string()]
    }
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct GitHubUserInfo {
    id: u64,
    email: Option<String>,
    name: Option<String>,
    login: String,
    avatar_url: Option<String>,
}

#[async_trait]
impl OAuthProvider for GitHubProvider {
    fn provider_name(&self) -> &str {
        "github"
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
            ("code", code),
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
                "failed to parse GitHub token response: {}",
                e
            )))
        })?;

        let user_body = self
            .http_client
            .get_with_bearer(USERINFO_URL, &token_resp.access_token)
            .await?;
        let gh_user: GitHubUserInfo = serde_json::from_str(&user_body).map_err(|e| {
            AppError::Auth(AuthError::OAuthProviderError(format!(
                "failed to parse GitHub user info: {}",
                e
            )))
        })?;

        // GitHub doesn't provide first/last name split, use name as-is
        Ok(OAuthUserInfo {
            provider: "github".to_string(),
            provider_user_id: gh_user.id.to_string(),
            email: gh_user.email,
            name: gh_user.name.clone(),
            first_name: gh_user.name,
            last_name: None,
            avatar_url: gh_user.avatar_url,
            raw_data: serde_json::to_value(&user_body).unwrap_or_default(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> OAuthConfig {
        OAuthConfig {
            client_id: "github-client-id".to_string(),
            client_secret: "github-secret".to_string(),
            redirect_url: "https://app.example.com/callback".to_string(),
            scopes: vec![],
        }
    }

    fn mock_http() -> Arc<dyn HttpClient> {
        Arc::new(crate::oauth::provider::tests::MockHttpClient::new())
    }

    #[test]
    fn github_authorization_url_correct() {
        let provider = GitHubProvider::new(test_config(), mock_http());
        let result = provider.authorization_url("test-state", None).unwrap();

        assert!(result.url.starts_with(AUTH_URL));
        assert!(result.url.contains("client_id=github-client-id"));
        assert!(result.url.contains("state=test-state"));
        assert!(result.url.contains("scope=read%3Auser+user%3Aemail"));
        assert!(result.url.contains("redirect_uri="));
        assert_eq!(result.state, "test-state");
    }
}
