use async_trait::async_trait;
use nucleus_core::error::AppError;
use serde::{Deserialize, Serialize};

/// Configuration for an OAuth provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthConfig {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_url: String,
    pub scopes: Vec<String>,
}

/// User info returned by an OAuth provider after a successful exchange.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthUserInfo {
    pub provider: String,
    pub provider_user_id: String,
    pub email: Option<String>,
    pub name: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub avatar_url: Option<String>,
    pub raw_data: serde_json::Value,
}

/// The authorization URL along with state and optional PKCE verifier.
#[derive(Debug)]
pub struct AuthorizationUrl {
    pub url: String,
    pub state: String,
    pub pkce_verifier: Option<String>,
}

/// A simple HTTP client trait so we can mock HTTP calls in tests
/// without pulling in reqwest as a heavy dependency.
#[async_trait]
pub trait HttpClient: Send + Sync {
    /// POST a form-encoded request and return the response body as a string.
    async fn post_form(&self, url: &str, params: &[(&str, &str)]) -> Result<String, AppError>;
    /// GET with a Bearer token and return the response body as a string.
    async fn get_with_bearer(&self, url: &str, token: &str) -> Result<String, AppError>;
}

/// The core trait that every OAuth provider must implement.
#[async_trait]
pub trait OAuthProvider: Send + Sync {
    /// Returns the provider name (e.g. "google", "github").
    fn provider_name(&self) -> &str;

    /// Generate the authorization URL for the OAuth flow.
    fn authorization_url(
        &self,
        state: &str,
        pkce_challenge: Option<&str>,
    ) -> Result<AuthorizationUrl, AppError>;

    /// Exchange the authorization code for user info.
    async fn exchange_code(
        &self,
        code: &str,
        pkce_verifier: Option<&str>,
    ) -> Result<OAuthUserInfo, AppError>;
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use nucleus_core::error::AuthError;
    use std::collections::HashMap;
    use std::sync::Mutex;

    /// A mock HTTP client for testing OAuth providers.
    pub struct MockHttpClient {
        post_responses: Mutex<HashMap<String, String>>,
        get_responses: Mutex<HashMap<String, String>>,
    }

    impl Default for MockHttpClient {
        fn default() -> Self {
            Self::new()
        }
    }

    impl MockHttpClient {
        pub fn new() -> Self {
            Self {
                post_responses: Mutex::new(HashMap::new()),
                get_responses: Mutex::new(HashMap::new()),
            }
        }

        pub fn with_post_response(self, url: &str, response: &str) -> Self {
            self.post_responses
                .lock()
                .unwrap()
                .insert(url.to_string(), response.to_string());
            self
        }

        pub fn with_get_response(self, url: &str, response: &str) -> Self {
            self.get_responses
                .lock()
                .unwrap()
                .insert(url.to_string(), response.to_string());
            self
        }
    }

    #[async_trait]
    impl HttpClient for MockHttpClient {
        async fn post_form(&self, url: &str, _params: &[(&str, &str)]) -> Result<String, AppError> {
            self.post_responses
                .lock()
                .unwrap()
                .get(url)
                .cloned()
                .ok_or_else(|| {
                    AppError::Auth(AuthError::OAuthProviderError(format!(
                        "no mock response for POST {}",
                        url
                    )))
                })
        }

        async fn get_with_bearer(&self, url: &str, _token: &str) -> Result<String, AppError> {
            self.get_responses
                .lock()
                .unwrap()
                .get(url)
                .cloned()
                .ok_or_else(|| {
                    AppError::Auth(AuthError::OAuthProviderError(format!(
                        "no mock response for GET {}",
                        url
                    )))
                })
        }
    }
}
