use std::sync::Arc;

use crate::core::error::{AppError, AuthError};
use async_trait::async_trait;
use serde::Deserialize;

use super::provider::{AuthorizationUrl, HttpClient, OAuthConfig, OAuthProvider, OAuthUserInfo};

const AUTH_URL: &str = "https://discord.com/oauth2/authorize";
const TOKEN_URL: &str = "https://discord.com/api/oauth2/token";
const USERINFO_URL: &str = "https://discord.com/api/users/@me";

pub struct DiscordProvider {
    config: OAuthConfig,
    http_client: Arc<dyn HttpClient>,
}

impl DiscordProvider {
    pub fn new(config: OAuthConfig, http_client: Arc<dyn HttpClient>) -> Self {
        Self {
            config,
            http_client,
        }
    }

    pub fn default_scopes() -> Vec<String> {
        vec!["identify".to_string(), "email".to_string()]
    }
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
}

#[derive(Debug, Deserialize)]
struct DiscordUserInfo {
    id: String,
    email: Option<String>,
    username: String,
    global_name: Option<String>,
    avatar: Option<String>,
}

impl DiscordUserInfo {
    fn avatar_url(&self) -> Option<String> {
        self.avatar.as_ref().map(|hash| {
            format!(
                "https://cdn.discordapp.com/avatars/{}/{}.png",
                self.id, hash
            )
        })
    }
}

#[async_trait]
impl OAuthProvider for DiscordProvider {
    fn provider_name(&self) -> &str {
        "discord"
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
                "failed to parse Discord token response: {}",
                e
            )))
        })?;

        let user_body = self
            .http_client
            .get_with_bearer(USERINFO_URL, &token_resp.access_token)
            .await?;
        let discord_user: DiscordUserInfo = serde_json::from_str(&user_body).map_err(|e| {
            AppError::Auth(AuthError::OAuthProviderError(format!(
                "failed to parse Discord user info: {}",
                e
            )))
        })?;

        let avatar_url = discord_user.avatar_url();

        Ok(OAuthUserInfo {
            provider: "discord".to_string(),
            provider_user_id: discord_user.id,
            email: discord_user.email,
            name: discord_user
                .global_name
                .clone()
                .or(Some(discord_user.username)),
            first_name: discord_user.global_name,
            last_name: None,
            avatar_url,
            raw_data: serde_json::to_value(&user_body).unwrap_or_default(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> OAuthConfig {
        OAuthConfig {
            client_id: "discord-client-id".to_string(),
            client_secret: "discord-secret".to_string(),
            redirect_url: "https://app.example.com/callback".to_string(),
            scopes: vec![],
        }
    }

    fn mock_http() -> Arc<dyn HttpClient> {
        Arc::new(crate::auth::oauth::provider::tests::MockHttpClient::new())
    }

    #[test]
    fn discord_authorization_url_correct() {
        let provider = DiscordProvider::new(test_config(), mock_http());
        let result = provider.authorization_url("test-state", None).unwrap();

        assert!(result.url.starts_with(AUTH_URL));
        assert!(result.url.contains("client_id=discord-client-id"));
        assert!(result.url.contains("state=test-state"));
        assert!(result.url.contains("scope=identify+email"));
        assert_eq!(result.state, "test-state");
    }

    #[test]
    fn discord_authorization_url_with_pkce() {
        let provider = DiscordProvider::new(test_config(), mock_http());
        let result = provider
            .authorization_url("state123", Some("challenge-value"))
            .unwrap();

        assert!(result.url.contains("code_challenge=challenge-value"));
        assert!(result.url.contains("code_challenge_method=S256"));
    }

    #[tokio::test]
    async fn discord_exchange_code_success() {
        let mock = crate::auth::oauth::provider::tests::MockHttpClient::new()
            .with_post_response(TOKEN_URL, r#"{"access_token":"discord-token-123"}"#)
            .with_get_response(
                USERINFO_URL,
                r#"{"id":"99999","email":"user@discord.com","username":"testuser","global_name":"Test User","avatar":"abc123"}"#,
            );

        let provider = DiscordProvider::new(test_config(), Arc::new(mock));
        let result = provider.exchange_code("auth-code", None).await.unwrap();

        assert_eq!(result.provider, "discord");
        assert_eq!(result.provider_user_id, "99999");
        assert_eq!(result.email, Some("user@discord.com".to_string()));
        assert_eq!(result.name, Some("Test User".to_string()));
        assert_eq!(
            result.avatar_url,
            Some("https://cdn.discordapp.com/avatars/99999/abc123.png".to_string())
        );
    }
}
