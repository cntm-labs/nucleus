use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::service::AuthService;
use nucleus_core::error::AppError;
use nucleus_core::types::ProjectId;

#[derive(Debug, Deserialize)]
pub struct SignUpRequest {
    pub email: String,
    pub password: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SignUpResponse {
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

/// POST /api/v1/auth/sign-up
///
/// Creates a new user with email and password, returns the user and a JWT.
/// In production the project_id will come from API key middleware.
pub async fn handle_sign_up(
    State(auth_service): State<Arc<AuthService>>,
    Json(req): Json<SignUpRequest>,
) -> Result<(StatusCode, Json<SignUpResponse>), AppError> {
    // TODO: project_id will come from middleware (API key extraction)
    let project_id = ProjectId::new();

    let (user, jwt) = auth_service
        .sign_up(
            &project_id,
            &req.email,
            &req.password,
            req.first_name,
            req.last_name,
        )
        .await?;

    let response = SignUpResponse {
        user: UserResponse {
            id: user.id.to_string(),
            email: user.email,
            first_name: user.first_name,
            last_name: user.last_name,
            created_at: user.created_at.to_rfc3339(),
        },
        jwt,
    };

    Ok((StatusCode::CREATED, Json(response)))
}
