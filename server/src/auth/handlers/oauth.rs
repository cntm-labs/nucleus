use std::collections::HashMap;
use std::sync::Arc;

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::auth::oauth::provider::{OAuthProvider, OAuthUserInfo};
use crate::auth::service::AuthService;
use crate::core::error::{AppError, AuthError};
use crate::core::types::ProjectId;
use crate::session::DeviceInfo;
use async_trait::async_trait;
use chrono::{DateTime, Utc};

// ---------------------------------------------------------------------------
// State & Storage Traits
// ---------------------------------------------------------------------------

pub struct OAuthHandlerState {
    pub providers: HashMap<String, Arc<dyn OAuthProvider>>,
    pub state_store: Arc<dyn OAuthStateStore>,
    pub auth_service: Arc<AuthService>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthStateData {
    pub project_id: ProjectId,
    pub provider: String,
    pub redirect_url: String,
    pub pkce_verifier: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[async_trait]
pub trait OAuthStateStore: Send + Sync {
    async fn store_state(
        &self,
        state: &str,
        data: &OAuthStateData,
        ttl_secs: u64,
    ) -> Result<(), AppError>;
    async fn consume_state(&self, state: &str) -> Result<Option<OAuthStateData>, AppError>;
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct OAuthStartRequest {
    pub provider: String,
    pub redirect_url: String,
}

#[derive(Debug, Serialize)]
pub struct OAuthStartResponse {
    pub url: String,
}

pub async fn handle_oauth_start(
    State(oauth_state): State<Arc<OAuthHandlerState>>,
    project_id: ProjectId,
    Json(req): Json<OAuthStartRequest>,
) -> Result<(StatusCode, Json<OAuthStartResponse>), AppError> {
    let provider = oauth_state
        .providers
        .get(&req.provider)
        .ok_or_else(|| AppError::Auth(AuthError::OAuthProviderNotFound(req.provider.clone())))?;

    let state = crate::core::crypto::generate_token();
    let pkce_verifier = crate::core::crypto::generate_token();
    let pkce_challenge = crate::core::crypto::generate_token_hash(&pkce_verifier);

    let auth_url = provider.authorization_url(&state, Some(&pkce_challenge))?;

    oauth_state
        .state_store
        .store_state(
            &state,
            &OAuthStateData {
                project_id,
                provider: req.provider,
                redirect_url: req.redirect_url,
                pkce_verifier: Some(pkce_verifier),
                created_at: Utc::now(),
            },
            900,
        )
        .await?;

    Ok((
        StatusCode::OK,
        Json(OAuthStartResponse { url: auth_url.url }),
    ))
}

#[derive(Debug, Deserialize)]
pub struct OAuthCallbackParams {
    pub code: String,
    pub state: String,
}

#[derive(Debug, Serialize)]
pub struct OAuthCallbackResponse {
    pub user: serde_json::Value,
    pub jwt: String,
    pub session_token: String,
}

pub async fn handle_oauth_callback(
    State(oauth_state): State<Arc<OAuthHandlerState>>,
    Query(params): Query<OAuthCallbackParams>,
) -> Result<(StatusCode, Json<OAuthCallbackResponse>), AppError> {
    let state_data = oauth_state
        .state_store
        .consume_state(&params.state)
        .await?
        .ok_or(AppError::Auth(AuthError::OAuthStateMismatch))?;

    let provider = oauth_state
        .providers
        .get(&state_data.provider)
        .ok_or_else(|| {
            AppError::Auth(AuthError::OAuthProviderNotFound(
                state_data.provider.clone(),
            ))
        })?;

    let user_info = provider
        .exchange_code(&params.code, state_data.pkce_verifier.as_deref())
        .await?;

    let (user, jwt, session_token, _is_new) =
        handle_oauth_user(&oauth_state, &state_data.project_id, &user_info).await?;

    Ok((
        StatusCode::OK,
        Json(OAuthCallbackResponse {
            user: serde_json::to_value(&user)
                .map_err(|e| AppError::Internal(anyhow::anyhow!("serialize error: {}", e)))?,
            jwt,
            session_token,
        }),
    ))
}

async fn handle_oauth_user(
    oauth_state: &OAuthHandlerState,
    project_id: &ProjectId,
    user_info: &OAuthUserInfo,
) -> Result<(crate::db::repos::user_repo::User, String, String, bool), AppError> {
    use crate::db::repos::credential_repo::NewCredential;
    use crate::db::repos::user_repo::NewUser;

    let auth_service = &oauth_state.auth_service;

    // Try to find existing user by OAuth provider credential
    let existing_cred = auth_service
        .credential_repo
        .find_by_provider_identifier("oauth", &user_info.provider, &user_info.provider_user_id)
        .await?;

    if let Some(cred) = existing_cred {
        let user = auth_service
            .user_repo
            .find_by_id(project_id, &cred.user_id)
            .await?
            .ok_or(AppError::Internal(anyhow::anyhow!(
                "user not found for existing OAuth credential"
            )))?;

        let (_session_token, session) = auth_service
            .session_service
            .create_session(&user.id, project_id, DeviceInfo::default(), 3600)
            .await?;

        let jwt = auth_service.issue_jwt_for_user(&user, project_id, &session.id)?;
        return Ok((user, jwt, session.id.to_string(), false));
    }

    if let Some(ref email) = user_info.email {
        if let Some(user) = auth_service
            .user_repo
            .find_by_email(project_id, email)
            .await?
        {
            let new_cred = NewCredential {
                user_id: user.id,
                credential_type: "oauth".to_string(),
                identifier: Some(user_info.provider_user_id.clone()),
                secret_hash: None,
                provider: Some(user_info.provider.clone()),
                provider_data: Some(user_info.raw_data.clone()),
            };
            auth_service.credential_repo.create(&new_cred).await?;

            let (_session_token, session) = auth_service
                .session_service
                .create_session(&user.id, project_id, DeviceInfo::default(), 3600)
                .await?;

            let jwt = auth_service.issue_jwt_for_user(&user, project_id, &session.id)?;
            return Ok((user, jwt, session.id.to_string(), false));
        }
    }

    let new_user = NewUser {
        email: user_info.email.clone().unwrap_or_default(),
        username: None,
        first_name: user_info.first_name.clone(),
        last_name: user_info.last_name.clone(),
        external_id: None,
        phone: None,
        avatar_url: user_info.avatar_url.clone(),
        metadata: None,
    };
    let user = auth_service.user_repo.create(project_id, &new_user).await?;

    let new_cred = NewCredential {
        user_id: user.id,
        credential_type: "oauth".to_string(),
        identifier: Some(user_info.provider_user_id.clone()),
        secret_hash: None,
        provider: Some(user_info.provider.clone()),
        provider_data: Some(user_info.raw_data.clone()),
    };
    auth_service.credential_repo.create(&new_cred).await?;

    let (_session_token, session) = auth_service
        .session_service
        .create_session(&user.id, project_id, DeviceInfo::default(), 3600)
        .await?;

    let jwt = auth_service.issue_jwt_for_user(&user, project_id, &session.id)?;
    Ok((user, jwt, session.id.to_string(), true))
}

#[cfg(test)]
#[allow(unused)]
#[allow(dead_code)]
mod tests {
    use super::*;
    use crate::auth::jwt::{JwtService, SigningKeyPair};
    use crate::auth::oauth::provider::{AuthorizationUrl, OAuthProvider, OAuthUserInfo};
    use crate::core::error::AppError;
    use crate::core::pagination::{PaginatedResponse, PaginationParams};
    use crate::core::types::*;
    use crate::db::repos::audit_repo::*;
    use crate::db::repos::credential_repo::*;
    use crate::db::repos::user_repo::*;
    use async_trait::async_trait;
    use chrono::Utc;
    use std::sync::Mutex;

    struct MockStateStore {
        states: Mutex<HashMap<String, OAuthStateData>>,
    }

    impl MockStateStore {
        fn new() -> Self {
            Self {
                states: Mutex::new(HashMap::new()),
            }
        }
    }

    #[async_trait]
    impl OAuthStateStore for MockStateStore {
        async fn store_state(
            &self,
            state: &str,
            data: &OAuthStateData,
            _ttl_secs: u64,
        ) -> Result<(), AppError> {
            self.states
                .lock()
                .unwrap()
                .insert(state.to_string(), data.clone());
            Ok(())
        }

        async fn consume_state(&self, state: &str) -> Result<Option<OAuthStateData>, AppError> {
            Ok(self.states.lock().unwrap().remove(state))
        }
    }

    struct MockProvider;

    #[async_trait]
    impl OAuthProvider for MockProvider {
        fn provider_name(&self) -> &str {
            "mock"
        }

        fn authorization_url(
            &self,
            state: &str,
            _pkce: Option<&str>,
        ) -> Result<AuthorizationUrl, AppError> {
            Ok(AuthorizationUrl {
                url: "https://mock.com/auth".to_string(),
                state: state.to_string(),
                pkce_verifier: None,
            })
        }

        async fn exchange_code(
            &self,
            _code: &str,
            _pkce: Option<&str>,
        ) -> Result<OAuthUserInfo, AppError> {
            Ok(OAuthUserInfo {
                provider: "mock".to_string(),
                provider_user_id: "user_123".to_string(),
                email: Some("test@example.com".to_string()),
                name: Some("Mock User".to_string()),
                first_name: None,
                last_name: None,
                avatar_url: None,
                raw_data: serde_json::json!({}),
            })
        }
    }

    #[tokio::test]
    async fn oauth_start_generates_url() {
        // Test placeholder
    }
}
