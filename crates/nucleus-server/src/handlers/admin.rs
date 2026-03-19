use axum::Json;
use nucleus_core::error::AppError;

// ---------------------------------------------------------------------------
// Phase 4: Admin routes (thin wrappers / direct delegation)
// ---------------------------------------------------------------------------

/// GET /api/v1/admin/users
pub async fn handle_admin_list_users() -> Result<Json<serde_json::Value>, AppError> {
    nucleus_identity::handlers::admin::handle_list_users().await
}

/// POST /api/v1/admin/users
pub async fn handle_admin_create_user(
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    nucleus_identity::handlers::admin::handle_create_user(Json(req)).await
}

/// GET /api/v1/admin/users/:id
pub async fn handle_admin_get_user() -> Result<Json<serde_json::Value>, AppError> {
    nucleus_identity::handlers::admin::handle_get_user().await
}

/// PATCH /api/v1/admin/users/:id
pub async fn handle_admin_update_user(
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    nucleus_identity::handlers::admin::handle_update_user(Json(req)).await
}

/// DELETE /api/v1/admin/users/:id
pub async fn handle_admin_delete_user() -> Result<axum::http::StatusCode, AppError> {
    nucleus_identity::handlers::admin::handle_delete_user().await
}

/// POST /api/v1/admin/users/:id/ban
pub async fn handle_admin_ban_user() -> Result<axum::http::StatusCode, AppError> {
    nucleus_identity::handlers::admin::handle_ban_user().await
}

/// POST /api/v1/admin/users/:id/unban
pub async fn handle_admin_unban_user() -> Result<axum::http::StatusCode, AppError> {
    nucleus_identity::handlers::admin::handle_unban_user().await
}
