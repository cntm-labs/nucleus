use std::sync::Arc;

use crate::core::error::{AppError, AuthError};
use async_trait::async_trait;
use serde::Deserialize;

use super::provider::{AuthorizationUrl, HttpClient, OAuthConfig, OAuthProvider, OAuthUserInfo};

const AUTH_URL: &str = "https://login.microsoftonline.com/common/oauth2/v2.0/authorize";
const TOKEN_URL: &str = "https://login.microsoftonline.com/common/oauth2/v2.0/token";
const USERINFO_URL: &str = "https://graph.microsoft.com/v1.0/me";

pub struct MicrosoftProvider {
    config: OAuthConfig,
    http_client: Arc<dyn HttpClient>,
}

impl MicrosoftProvider {
    pub fn new(config: OAuthConfig, http_client: Arc<dyn HttpClient>) -> Self {
        Self {
            config,
            http_client,
        }
    }

    /// Return default scopes for Microsoft OAuth.
    pub fn default_scopes() -> Vec<String> {
        vec![
            "openid".to_string(),
            "email".to_string(),
            "profile".to_string(),
        ]
    }
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MicrosoftUserInfo {
    id: String,
    mail: Option<String>,
    user_principal_name: Option<String>,
    display_name: Option<String>,
    given_name: Option<String>,
    surname: Option<String>,
}

#[async_trait]
impl OAuthProvider for MicrosoftProvider {
    fn provider_name(&self) -> &str {
        "microsoft"
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
                "failed to parse Microsoft token response: {}",
                e
            )))
        })?;

        let user_body = self
            .http_client
            .get_with_bearer(USERINFO_URL, &token_resp.access_token)
            .await?;
        let ms_user: MicrosoftUserInfo = serde_json::from_str(&user_body).map_err(|e| {
            AppError::Auth(AuthError::OAuthProviderError(format!(
                "failed to parse Microsoft user info: {}",
                e
            )))
        })?;

        // Microsoft uses `mail` or falls back to `userPrincipalName` for email
        let email = ms_user.mail.or(ms_user.user_principal_name);

        Ok(OAuthUserInfo {
            provider: "microsoft".to_string(),
            provider_user_id: ms_user.id,
            email,
            name: ms_user.display_name,
            first_name: ms_user.given_name,
            last_name: ms_user.surname,
            avatar_url: None,
            raw_data: serde_json::to_value(&user_body).unwrap_or_default(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> OAuthConfig {
        OAuthConfig {
            client_id: "ms-client-id".to_string(),
            client_secret: "ms-secret".to_string(),
            redirect_url: "https://app.example.com/callback".to_string(),
            scopes: vec![],
        }
    }

    fn mock_http() -> Arc<dyn HttpClient> {
        Arc::new(crate::auth::oauth::provider::tests::MockHttpClient::new())
    }

    #[test]
    fn microsoft_authorization_url_correct() {
        let provider = MicrosoftProvider::new(test_config(), mock_http());
        let result = provider.authorization_url("test-state", None).unwrap();

        assert!(result.url.starts_with(AUTH_URL));
        assert!(result.url.contains("client_id=ms-client-id"));
        assert!(result.url.contains("state=test-state"));
        assert!(result.url.contains("response_type=code"));
        assert!(result.url.contains("scope=openid+email+profile"));
        assert!(result.url.contains("redirect_uri="));
        assert_eq!(result.state, "test-state");
    }
}
