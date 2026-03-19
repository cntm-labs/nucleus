use std::collections::HashMap;
use std::sync::Arc;

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};

use nucleus_core::error::{AppError, AuthError};
use nucleus_core::types::ProjectId;

use crate::oauth::provider::{OAuthProvider, OAuthUserInfo};
use crate::pkce;

// ---------------------------------------------------------------------------
// OAuth State Store trait (backed by Redis in production)
// ---------------------------------------------------------------------------

/// Trait for storing OAuth state (CSRF protection) with TTL.
#[async_trait::async_trait]
pub trait OAuthStateStore: Send + Sync {
    /// Store an OAuth state value with associated data and a TTL in seconds.
    async fn store_state(
        &self,
        state: &str,
        data: &OAuthStateData,
        ttl_secs: u64,
    ) -> Result<(), AppError>;

    /// Retrieve and delete an OAuth state value (single use).
    async fn consume_state(&self, state: &str) -> Result<Option<OAuthStateData>, AppError>;
}

/// Data stored alongside the OAuth state for CSRF protection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthStateData {
    pub provider: String,
    pub pkce_verifier: Option<String>,
    pub redirect_url: Option<String>,
    pub project_id: String,
}

// ---------------------------------------------------------------------------
// Request / Response types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct OAuthStartRequest {
    pub provider: String,
    pub redirect_url: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct OAuthStartResponse {
    pub authorization_url: String,
}

#[derive(Debug, Deserialize)]
pub struct OAuthCallbackParams {
    pub code: String,
    pub state: String,
}

#[derive(Debug, Serialize)]
pub struct OAuthCallbackResponse {
    pub user: OAuthUserResponse,
    pub jwt: String,
    pub is_new_user: bool,
}

#[derive(Debug, Serialize)]
pub struct OAuthUserResponse {
    pub id: String,
    pub email: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}

// ---------------------------------------------------------------------------
// App state for OAuth handlers
// ---------------------------------------------------------------------------

/// Shared state for OAuth handlers.
pub struct OAuthHandlerState {
    pub providers: HashMap<String, Arc<dyn OAuthProvider>>,
    pub state_store: Arc<dyn OAuthStateStore>,
    pub auth_service: Arc<crate::service::AuthService>,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// POST /auth/sign-in/oauth
///
/// Begins an OAuth flow. Generates state + PKCE, stores in Redis,
/// and returns the authorization URL.
pub async fn handle_oauth_start(
    State(oauth_state): State<Arc<OAuthHandlerState>>,
    Json(req): Json<OAuthStartRequest>,
) -> Result<(StatusCode, Json<OAuthStartResponse>), AppError> {
    // 1. Find the provider
    let provider = oauth_state
        .providers
        .get(&req.provider)
        .ok_or_else(|| {
            AppError::Auth(AuthError::OAuthProviderError(format!(
                "unsupported provider: {}",
                req.provider
            )))
        })?;

    // 2. Generate PKCE verifier + challenge
    let verifier = pkce::generate_verifier();
    let challenge = pkce::generate_challenge(&verifier);

    // 3. Generate random state for CSRF protection
    let state = nucleus_core::crypto::generate_token();

    // 4. Build authorization URL
    let auth_url = provider.authorization_url(&state, Some(&challenge))?;

    // 5. Store state in Redis with 10-minute TTL
    // TODO: project_id will come from middleware in production
    let project_id = ProjectId::new();
    let state_data = OAuthStateData {
        provider: req.provider,
        pkce_verifier: Some(verifier),
        redirect_url: req.redirect_url,
        project_id: project_id.to_string(),
    };
    oauth_state
        .state_store
        .store_state(&state, &state_data, 600)
        .await?;

    Ok((
        StatusCode::OK,
        Json(OAuthStartResponse {
            authorization_url: auth_url.url,
        }),
    ))
}

/// GET /auth/oauth/callback
///
/// Handles the OAuth callback. Validates state, exchanges code for user info,
/// creates/links the user, and returns a JWT.
pub async fn handle_oauth_callback(
    State(oauth_state): State<Arc<OAuthHandlerState>>,
    Query(params): Query<OAuthCallbackParams>,
) -> Result<(StatusCode, Json<OAuthCallbackResponse>), AppError> {
    // 1. Consume state from Redis (single-use)
    let state_data = oauth_state
        .state_store
        .consume_state(&params.state)
        .await?
        .ok_or(AppError::Auth(AuthError::OAuthStateMismatch))?;

    // 2. Find the provider
    let provider = oauth_state
        .providers
        .get(&state_data.provider)
        .ok_or_else(|| {
            AppError::Auth(AuthError::OAuthProviderError(format!(
                "unsupported provider: {}",
                state_data.provider
            )))
        })?;

    // 3. Exchange code for user info
    let user_info = provider
        .exchange_code(&params.code, state_data.pkce_verifier.as_deref())
        .await?;

    // 4. Create or link user (delegate to auth service)
    let project_id: ProjectId = state_data
        .project_id
        .parse()
        .map_err(|_| AppError::Internal(anyhow::anyhow!("invalid project_id in state")))?;

    let (user, jwt, is_new_user) = create_or_link_user(
        &oauth_state.auth_service,
        &project_id,
        &user_info,
    )
    .await?;

    Ok((
        StatusCode::OK,
        Json(OAuthCallbackResponse {
            user: OAuthUserResponse {
                id: user.id.to_string(),
                email: Some(user.email),
                first_name: user.first_name,
                last_name: user.last_name,
            },
            jwt,
            is_new_user,
        }),
    ))
}

/// Create a new user from OAuth info, or link to an existing user if the email
/// already exists.
async fn create_or_link_user(
    auth_service: &crate::service::AuthService,
    project_id: &ProjectId,
    user_info: &OAuthUserInfo,
) -> Result<(nucleus_db::repos::user_repo::User, String, bool), AppError> {
    use nucleus_db::repos::credential_repo::NewCredential;
    use nucleus_db::repos::user_repo::NewUser;

    // Try to find existing user by OAuth provider credential
    let existing_cred = auth_service
        .credential_repo()
        .find_by_provider_identifier("oauth", &user_info.provider, &user_info.provider_user_id)
        .await?;

    if let Some(cred) = existing_cred {
        // User already linked — just find them and issue JWT
        let user = auth_service
            .user_repo()
            .find_by_id(project_id, &cred.user_id)
            .await?
            .ok_or(AppError::Internal(anyhow::anyhow!(
                "user not found for existing OAuth credential"
            )))?;
        let jwt = auth_service.issue_jwt_for_user(&user, project_id)?;
        return Ok((user, jwt, false));
    }

    // Check if user with same email exists (link)
    if let Some(ref email) = user_info.email {
        if let Some(user) = auth_service
            .user_repo()
            .find_by_email(project_id, email)
            .await?
        {
            // Link OAuth credential to existing user
            let new_cred = NewCredential {
                user_id: user.id,
                credential_type: "oauth".to_string(),
                identifier: Some(user_info.provider_user_id.clone()),
                secret_hash: None,
                provider: Some(user_info.provider.clone()),
                provider_data: Some(user_info.raw_data.clone()),
            };
            auth_service.credential_repo().create(&new_cred).await?;
            let jwt = auth_service.issue_jwt_for_user(&user, project_id)?;
            return Ok((user, jwt, false));
        }
    }

    // Create new user
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
    let user = auth_service.user_repo().create(project_id, &new_user).await?;

    // Create OAuth credential
    let new_cred = NewCredential {
        user_id: user.id,
        credential_type: "oauth".to_string(),
        identifier: Some(user_info.provider_user_id.clone()),
        secret_hash: None,
        provider: Some(user_info.provider.clone()),
        provider_data: Some(user_info.raw_data.clone()),
    };
    auth_service.credential_repo().create(&new_cred).await?;

    let jwt = auth_service.issue_jwt_for_user(&user, project_id)?;
    Ok((user, jwt, true))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jwt::{JwtService, SigningKeyPair};
    use crate::oauth::provider::{AuthorizationUrl, OAuthProvider, OAuthUserInfo};
    use async_trait::async_trait;
    use chrono::Utc;
    use nucleus_core::error::AppError;
    use nucleus_core::pagination::{PaginatedResponse, PaginationParams};
    use nucleus_core::types::*;
    use nucleus_db::repos::audit_repo::*;
    use nucleus_db::repos::credential_repo::*;
    use nucleus_db::repos::user_repo::*;
    use std::sync::Mutex;

    // -- Mock OAuthStateStore ------------------------------------------------

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

        async fn consume_state(
            &self,
            state: &str,
        ) -> Result<Option<OAuthStateData>, AppError> {
            Ok(self.states.lock().unwrap().remove(state))
        }
    }

    // -- Mock OAuthProvider --------------------------------------------------

    struct MockOAuthProvider {
        name: String,
        user_info: OAuthUserInfo,
    }

    impl MockOAuthProvider {
        fn new(name: &str, user_info: OAuthUserInfo) -> Self {
            Self {
                name: name.to_string(),
                user_info,
            }
        }
    }

    #[async_trait]
    impl OAuthProvider for MockOAuthProvider {
        fn provider_name(&self) -> &str {
            &self.name
        }

        fn authorization_url(
            &self,
            state: &str,
            _pkce_challenge: Option<&str>,
        ) -> Result<AuthorizationUrl, AppError> {
            Ok(AuthorizationUrl {
                url: format!("https://provider.example.com/auth?state={}", state),
                state: state.to_string(),
                pkce_verifier: None,
            })
        }

        async fn exchange_code(
            &self,
            _code: &str,
            _pkce_verifier: Option<&str>,
        ) -> Result<OAuthUserInfo, AppError> {
            Ok(self.user_info.clone())
        }
    }

    // -- Mock repos (reuse pattern from service.rs tests) --------------------

    struct MockUserRepo {
        users: Mutex<Vec<User>>,
    }

    impl MockUserRepo {
        fn new() -> Self {
            Self {
                users: Mutex::new(Vec::new()),
            }
        }

        fn with_user(user: User) -> Self {
            Self {
                users: Mutex::new(vec![user]),
            }
        }
    }

    #[async_trait]
    impl UserRepository for MockUserRepo {
        async fn create(&self, project_id: &ProjectId, new: &NewUser) -> Result<User, AppError> {
            let user = User {
                id: UserId::new(),
                project_id: *project_id,
                external_id: new.external_id.clone(),
                email: new.email.clone(),
                email_verified: false,
                phone: new.phone.clone(),
                phone_verified: false,
                username: new.username.clone(),
                first_name: new.first_name.clone(),
                last_name: new.last_name.clone(),
                avatar_url: new.avatar_url.clone(),
                metadata: new
                    .metadata
                    .clone()
                    .unwrap_or(serde_json::Value::Object(Default::default())),
                private_metadata: serde_json::Value::Object(Default::default()),
                last_sign_in_at: None,
                banned_at: None,
                deleted_at: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };
            self.users.lock().unwrap().push(user.clone());
            Ok(user)
        }

        async fn find_by_id(
            &self,
            project_id: &ProjectId,
            user_id: &UserId,
        ) -> Result<Option<User>, AppError> {
            let users = self.users.lock().unwrap();
            Ok(users
                .iter()
                .find(|u| u.project_id == *project_id && u.id == *user_id)
                .cloned())
        }

        async fn find_by_email(
            &self,
            project_id: &ProjectId,
            email: &str,
        ) -> Result<Option<User>, AppError> {
            let users = self.users.lock().unwrap();
            Ok(users
                .iter()
                .find(|u| u.project_id == *project_id && u.email == email)
                .cloned())
        }

        async fn find_by_username(
            &self,
            _project_id: &ProjectId,
            _username: &str,
        ) -> Result<Option<User>, AppError> {
            Ok(None)
        }

        async fn update(
            &self,
            _project_id: &ProjectId,
            _user_id: &UserId,
            _update: &UpdateUser,
        ) -> Result<User, AppError> {
            unimplemented!()
        }

        async fn soft_delete(
            &self,
            _project_id: &ProjectId,
            _user_id: &UserId,
        ) -> Result<(), AppError> {
            unimplemented!()
        }

        async fn list(
            &self,
            _project_id: &ProjectId,
            _params: &PaginationParams,
        ) -> Result<PaginatedResponse<User>, AppError> {
            unimplemented!()
        }

        async fn ban(
            &self,
            _project_id: &ProjectId,
            _user_id: &UserId,
        ) -> Result<(), AppError> {
            unimplemented!()
        }

        async fn unban(
            &self,
            _project_id: &ProjectId,
            _user_id: &UserId,
        ) -> Result<(), AppError> {
            unimplemented!()
        }
    }

    struct MockCredentialRepo {
        credentials: Mutex<Vec<Credential>>,
    }

    impl MockCredentialRepo {
        fn new() -> Self {
            Self {
                credentials: Mutex::new(Vec::new()),
            }
        }
    }

    #[async_trait]
    impl CredentialRepository for MockCredentialRepo {
        async fn create(&self, new: &NewCredential) -> Result<Credential, AppError> {
            let cred = Credential {
                id: CredentialId::new(),
                user_id: new.user_id,
                credential_type: new.credential_type.clone(),
                identifier: new.identifier.clone(),
                secret_hash: new.secret_hash.clone(),
                provider: new.provider.clone(),
                provider_data: new.provider_data.clone(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };
            self.credentials.lock().unwrap().push(cred.clone());
            Ok(cred)
        }

        async fn find_by_user_and_type(
            &self,
            user_id: &UserId,
            credential_type: &str,
        ) -> Result<Vec<Credential>, AppError> {
            let creds = self.credentials.lock().unwrap();
            Ok(creds
                .iter()
                .filter(|c| c.user_id == *user_id && c.credential_type == credential_type)
                .cloned()
                .collect())
        }

        async fn find_by_provider_identifier(
            &self,
            credential_type: &str,
            provider: &str,
            identifier: &str,
        ) -> Result<Option<Credential>, AppError> {
            let creds = self.credentials.lock().unwrap();
            Ok(creds
                .iter()
                .find(|c| {
                    c.credential_type == credential_type
                        && c.provider.as_deref() == Some(provider)
                        && c.identifier.as_deref() == Some(identifier)
                })
                .cloned())
        }

        async fn update_secret(
            &self,
            _id: &CredentialId,
            _new_secret_hash: &str,
        ) -> Result<(), AppError> {
            unimplemented!()
        }

        async fn delete(&self, _id: &CredentialId) -> Result<(), AppError> {
            unimplemented!()
        }
    }

    struct MockAuditRepo;

    #[async_trait]
    impl AuditRepository for MockAuditRepo {
        async fn create_audit_log(&self, _log: &NewAuditLog) -> Result<AuditLog, AppError> {
            unimplemented!()
        }

        async fn list_audit_logs(
            &self,
            _project_id: &ProjectId,
            _params: &PaginationParams,
        ) -> Result<PaginatedResponse<AuditLog>, AppError> {
            unimplemented!()
        }

        async fn create_sign_in_attempt(
            &self,
            attempt: &NewSignInAttempt,
        ) -> Result<SignInAttempt, AppError> {
            Ok(SignInAttempt {
                id: uuid::Uuid::new_v4(),
                project_id: attempt.project_id,
                user_id: attempt.user_id,
                method: attempt.method.clone(),
                status: attempt.status.clone(),
                failure_reason: attempt.failure_reason.clone(),
                ip: attempt.ip.clone(),
                user_agent: attempt.user_agent.clone(),
                country_code: attempt.country_code.clone(),
                city: attempt.city.clone(),
                created_at: Utc::now(),
            })
        }

        async fn list_sign_in_attempts(
            &self,
            _project_id: &ProjectId,
            _user_id: &UserId,
            _params: &PaginationParams,
        ) -> Result<PaginatedResponse<SignInAttempt>, AppError> {
            unimplemented!()
        }
    }

    // -- Helpers -------------------------------------------------------------

    fn test_signing_key() -> Arc<SigningKeyPair> {
        Arc::new(JwtService::generate_key_pair("test-kid").unwrap())
    }

    fn make_oauth_user_info() -> OAuthUserInfo {
        OAuthUserInfo {
            provider: "mock".to_string(),
            provider_user_id: "provider-user-123".to_string(),
            email: Some("oauth-user@example.com".to_string()),
            name: Some("OAuth User".to_string()),
            first_name: Some("OAuth".to_string()),
            last_name: Some("User".to_string()),
            avatar_url: None,
            raw_data: serde_json::json!({}),
        }
    }

    fn make_test_state(
        user_repo: Arc<dyn UserRepository>,
        cred_repo: Arc<dyn CredentialRepository>,
    ) -> Arc<OAuthHandlerState> {
        let audit_repo: Arc<dyn AuditRepository> = Arc::new(MockAuditRepo);
        let auth_service = Arc::new(crate::service::AuthService::new(
            user_repo,
            cred_repo,
            audit_repo,
            test_signing_key(),
            "https://nucleus.test".to_string(),
            3600,
        ));

        let user_info = make_oauth_user_info();
        let mock_provider: Arc<dyn OAuthProvider> =
            Arc::new(MockOAuthProvider::new("mock", user_info));
        let mut providers = HashMap::new();
        providers.insert("mock".to_string(), mock_provider);

        let state_store: Arc<dyn OAuthStateStore> = Arc::new(MockStateStore::new());

        Arc::new(OAuthHandlerState {
            providers,
            state_store,
            auth_service,
        })
    }

    // -- Tests ---------------------------------------------------------------

    #[tokio::test]
    async fn oauth_start_returns_authorization_url() {
        let user_repo: Arc<dyn UserRepository> = Arc::new(MockUserRepo::new());
        let cred_repo: Arc<dyn CredentialRepository> = Arc::new(MockCredentialRepo::new());
        let state = make_test_state(user_repo, cred_repo);

        let req = OAuthStartRequest {
            provider: "mock".to_string(),
            redirect_url: None,
        };

        let result = handle_oauth_start(State(state.clone()), Json(req)).await;
        assert!(result.is_ok());
        let (status, Json(resp)) = result.unwrap();
        assert_eq!(status, StatusCode::OK);
        assert!(resp
            .authorization_url
            .starts_with("https://provider.example.com/auth?state="));
    }

    #[tokio::test]
    async fn oauth_callback_creates_user_on_first_login() {
        let user_repo: Arc<dyn UserRepository> = Arc::new(MockUserRepo::new());
        let cred_repo: Arc<dyn CredentialRepository> = Arc::new(MockCredentialRepo::new());
        let handler_state = make_test_state(user_repo, cred_repo);

        // First, store a state manually
        let project_id = ProjectId::new();
        let state_data = OAuthStateData {
            provider: "mock".to_string(),
            pkce_verifier: Some("test-verifier".to_string()),
            redirect_url: None,
            project_id: project_id.to_string(),
        };
        handler_state
            .state_store
            .store_state("test-state-123", &state_data, 600)
            .await
            .unwrap();

        let params = OAuthCallbackParams {
            code: "auth-code-abc".to_string(),
            state: "test-state-123".to_string(),
        };

        let result =
            handle_oauth_callback(State(handler_state.clone()), Query(params)).await;
        assert!(result.is_ok());
        let (status, Json(resp)) = result.unwrap();
        assert_eq!(status, StatusCode::OK);
        assert!(resp.is_new_user);
        assert!(!resp.jwt.is_empty());
        assert_eq!(resp.user.email, Some("oauth-user@example.com".to_string()));
    }

    #[tokio::test]
    async fn oauth_callback_links_existing_user() {
        let project_id = ProjectId::new();

        // Pre-existing user with matching email
        let existing_user = User {
            id: UserId::new(),
            project_id,
            external_id: None,
            email: "oauth-user@example.com".to_string(),
            email_verified: true,
            phone: None,
            phone_verified: false,
            username: None,
            first_name: Some("Existing".to_string()),
            last_name: Some("User".to_string()),
            avatar_url: None,
            metadata: serde_json::Value::Object(Default::default()),
            private_metadata: serde_json::Value::Object(Default::default()),
            last_sign_in_at: None,
            banned_at: None,
            deleted_at: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let user_repo: Arc<dyn UserRepository> =
            Arc::new(MockUserRepo::with_user(existing_user.clone()));
        let cred_repo: Arc<dyn CredentialRepository> = Arc::new(MockCredentialRepo::new());
        let handler_state = make_test_state(user_repo, cred_repo);

        let state_data = OAuthStateData {
            provider: "mock".to_string(),
            pkce_verifier: Some("test-verifier".to_string()),
            redirect_url: None,
            project_id: project_id.to_string(),
        };
        handler_state
            .state_store
            .store_state("link-state", &state_data, 600)
            .await
            .unwrap();

        let params = OAuthCallbackParams {
            code: "auth-code-link".to_string(),
            state: "link-state".to_string(),
        };

        let result =
            handle_oauth_callback(State(handler_state.clone()), Query(params)).await;
        assert!(result.is_ok());
        let (status, Json(resp)) = result.unwrap();
        assert_eq!(status, StatusCode::OK);
        assert!(!resp.is_new_user);
        assert_eq!(resp.user.id, existing_user.id.to_string());
    }

    #[tokio::test]
    async fn oauth_callback_rejects_invalid_state() {
        let user_repo: Arc<dyn UserRepository> = Arc::new(MockUserRepo::new());
        let cred_repo: Arc<dyn CredentialRepository> = Arc::new(MockCredentialRepo::new());
        let handler_state = make_test_state(user_repo, cred_repo);

        let params = OAuthCallbackParams {
            code: "auth-code-abc".to_string(),
            state: "nonexistent-state".to_string(),
        };

        let result =
            handle_oauth_callback(State(handler_state), Query(params)).await;
        assert!(matches!(
            result,
            Err(AppError::Auth(AuthError::OAuthStateMismatch))
        ));
    }
}
