use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::auth::service::AuthService;
use crate::core::error::AppError;
use crate::core::types::ProjectId;

#[derive(Debug, Deserialize)]
pub struct SignInRequest {
    pub identifier: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct SignInResponse {
    pub user: UserResponse,
    pub jwt: String,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: String,
    pub email: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub created_at: String,
}

/// POST /api/v1/auth/sign-in
///
/// Authenticates a user with email and password, returns the user and a JWT.
/// In production the project_id will come from API key middleware.
pub async fn handle_sign_in(
    State(auth_service): State<Arc<AuthService>>,
    Json(req): Json<SignInRequest>,
) -> Result<(StatusCode, Json<SignInResponse>), AppError> {
    // TODO: project_id will come from middleware (API key extraction)
    let project_id = ProjectId::new();

    // TODO: extract ip and user_agent from request headers via middleware
    let (user, jwt) = auth_service
        .sign_in(&project_id, &req.identifier, &req.password, None, None)
        .await?;

    let response = SignInResponse {
        user: UserResponse {
            id: user.id.to_string(),
            email: user.email,
            first_name: user.first_name,
            last_name: user.last_name,
            created_at: user.created_at.to_rfc3339(),
        },
        jwt,
    };

    Ok((StatusCode::OK, Json(response)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::service::tests::{
        make_password_credential, make_service, MockAuditRepo, MockCredentialRepo,
    };
    use crate::core::pagination::{PaginatedResponse, PaginationParams};
    use crate::core::types::{ProjectId, UserId};
    use crate::db::repos::user_repo::{NewUser, UpdateUser, User, UserRepository};
    use async_trait::async_trait;
    use axum::extract::State;
    use chrono::Utc;
    use std::sync::Mutex;

    // Handler-level mock that ignores project_id (since handlers use ProjectId::new()).
    struct LenientMockUserRepo {
        users: Mutex<Vec<User>>,
    }

    impl LenientMockUserRepo {
        fn with_user(user: User) -> Self {
            Self {
                users: Mutex::new(vec![user]),
            }
        }
    }

    #[async_trait]
    impl UserRepository for LenientMockUserRepo {
        async fn create(&self, _project_id: &ProjectId, new: &NewUser) -> Result<User, AppError> {
            let user = User {
                id: UserId::new(),
                project_id: ProjectId::new(),
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
            _pid: &ProjectId,
            uid: &UserId,
        ) -> Result<Option<User>, AppError> {
            Ok(self
                .users
                .lock()
                .unwrap()
                .iter()
                .find(|u| u.id == *uid)
                .cloned())
        }
        async fn find_by_email(
            &self,
            _pid: &ProjectId,
            email: &str,
        ) -> Result<Option<User>, AppError> {
            Ok(self
                .users
                .lock()
                .unwrap()
                .iter()
                .find(|u| u.email == email)
                .cloned())
        }
        async fn find_by_username(
            &self,
            _pid: &ProjectId,
            _username: &str,
        ) -> Result<Option<User>, AppError> {
            Ok(None)
        }
        async fn update(
            &self,
            _pid: &ProjectId,
            _uid: &UserId,
            _upd: &UpdateUser,
        ) -> Result<User, AppError> {
            unimplemented!()
        }
        async fn soft_delete(&self, _pid: &ProjectId, _uid: &UserId) -> Result<(), AppError> {
            unimplemented!()
        }
        async fn list(
            &self,
            _pid: &ProjectId,
            _params: &PaginationParams,
        ) -> Result<PaginatedResponse<User>, AppError> {
            unimplemented!()
        }
        async fn ban(&self, _pid: &ProjectId, _uid: &UserId) -> Result<(), AppError> {
            unimplemented!()
        }
        async fn unban(&self, _pid: &ProjectId, _uid: &UserId) -> Result<(), AppError> {
            unimplemented!()
        }
    }

    fn build_auth_service_with_user() -> Arc<AuthService> {
        let project_id = ProjectId::new();
        let mut user = crate::auth::service::tests::make_test_user(&project_id);
        user.email = "existing@example.com".to_string();
        let cred = make_password_credential(&user.id, "SecurePass123!");

        let user_repo = Arc::new(LenientMockUserRepo::with_user(user));
        let cred_repo = Arc::new(MockCredentialRepo::with_credential(cred));
        let audit_repo = Arc::new(MockAuditRepo::new());
        Arc::new(make_service(user_repo, cred_repo, audit_repo))
    }

    #[tokio::test]
    async fn sign_in_success_returns_ok_with_jwt() {
        let service = build_auth_service_with_user();

        let req = SignInRequest {
            identifier: "existing@example.com".to_string(),
            password: "SecurePass123!".to_string(),
        };

        let result = handle_sign_in(State(service), Json(req)).await;
        assert!(result.is_ok());

        let (status, Json(response)) = result.unwrap();
        assert_eq!(status, StatusCode::OK);
        assert_eq!(response.user.email, "existing@example.com");
        assert_eq!(response.user.first_name, Some("Test".to_string()));
        assert!(!response.jwt.is_empty());
    }

    #[tokio::test]
    async fn sign_in_nonexistent_email_returns_error() {
        let service = build_auth_service_with_user();

        let req = SignInRequest {
            identifier: "nobody@example.com".to_string(),
            password: "SecurePass123!".to_string(),
        };

        let result = handle_sign_in(State(service), Json(req)).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn sign_in_wrong_password_returns_error() {
        let service = build_auth_service_with_user();

        let req = SignInRequest {
            identifier: "existing@example.com".to_string(),
            password: "WrongPassword!".to_string(),
        };

        let result = handle_sign_in(State(service), Json(req)).await;
        assert!(result.is_err());
    }
}
