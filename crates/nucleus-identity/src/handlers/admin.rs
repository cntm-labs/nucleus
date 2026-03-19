use axum::Json;
use nucleus_core::error::AppError;

/// GET /api/v1/admin/users -- list/search users (paginated)
pub async fn handle_list_users() -> Result<Json<serde_json::Value>, AppError> {
    todo!()
}

/// POST /api/v1/admin/users -- create user
pub async fn handle_create_user(
    Json(_req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    todo!()
}

/// GET /api/v1/admin/users/:id -- get user
pub async fn handle_get_user() -> Result<Json<serde_json::Value>, AppError> {
    todo!()
}

/// PATCH /api/v1/admin/users/:id -- update user
pub async fn handle_update_user(
    Json(_req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    todo!()
}

/// DELETE /api/v1/admin/users/:id -- soft delete
pub async fn handle_delete_user() -> Result<axum::http::StatusCode, AppError> {
    todo!()
}

/// POST /api/v1/admin/users/:id/ban
pub async fn handle_ban_user() -> Result<axum::http::StatusCode, AppError> {
    todo!()
}

/// POST /api/v1/admin/users/:id/unban
pub async fn handle_unban_user() -> Result<axum::http::StatusCode, AppError> {
    todo!()
}
