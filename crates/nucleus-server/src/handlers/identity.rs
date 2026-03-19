use axum::Json;
use nucleus_core::error::AppError;

// ---------------------------------------------------------------------------
// Phase 4: User profile routes (thin wrappers / direct delegation)
// ---------------------------------------------------------------------------

/// GET /api/v1/users/me
pub async fn handle_get_me() -> Result<Json<nucleus_identity::handlers::me::UserProfileResponse>, AppError> {
    nucleus_identity::handlers::me::handle_get_me().await
}

/// PATCH /api/v1/users/me
pub async fn handle_update_me(
    Json(req): Json<nucleus_identity::handlers::me::UpdateProfileRequest>,
) -> Result<Json<nucleus_identity::handlers::me::UserProfileResponse>, AppError> {
    nucleus_identity::handlers::me::handle_update_me(Json(req)).await
}

/// DELETE /api/v1/users/me
pub async fn handle_delete_me() -> Result<axum::http::StatusCode, AppError> {
    nucleus_identity::handlers::me::handle_delete_me().await
}

/// GET /api/v1/users/me/sessions
pub async fn handle_list_my_sessions() -> Result<Json<serde_json::Value>, AppError> {
    nucleus_identity::handlers::me::handle_list_my_sessions().await
}

/// DELETE /api/v1/users/me/sessions/:id
pub async fn handle_revoke_my_session() -> Result<axum::http::StatusCode, AppError> {
    nucleus_identity::handlers::me::handle_revoke_my_session().await
}
