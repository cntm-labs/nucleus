use axum::Json;
use nucleus_core::error::AppError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct UserProfileResponse {
    pub id: String,
    pub email: String,
    pub email_verified: bool,
    pub username: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub avatar_url: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: String,
    pub updated_at: String,
}

/// GET /api/v1/users/me
pub async fn handle_get_me() -> Result<Json<UserProfileResponse>, AppError> {
    // Extract user from JWT claims, call user_service.get_me()
    todo!()
}

#[derive(Debug, Deserialize)]
pub struct UpdateProfileRequest {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub username: Option<String>,
    pub avatar_url: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

/// PATCH /api/v1/users/me
pub async fn handle_update_me(
    Json(_req): Json<UpdateProfileRequest>,
) -> Result<Json<UserProfileResponse>, AppError> {
    todo!()
}

/// DELETE /api/v1/users/me
pub async fn handle_delete_me() -> Result<axum::http::StatusCode, AppError> {
    todo!()
}

/// GET /api/v1/users/me/sessions
pub async fn handle_list_my_sessions() -> Result<Json<serde_json::Value>, AppError> {
    todo!()
}

/// DELETE /api/v1/users/me/sessions/:id
pub async fn handle_revoke_my_session() -> Result<axum::http::StatusCode, AppError> {
    todo!()
}
