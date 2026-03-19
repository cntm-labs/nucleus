use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::service::AuthService;
use nucleus_core::error::AppError;
use nucleus_core::types::ProjectId;

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
