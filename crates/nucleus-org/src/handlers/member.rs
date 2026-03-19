use axum::Json;
use nucleus_core::error::AppError;

/// GET /api/v1/orgs/:slug/members
pub async fn handle_list_members() -> Result<Json<serde_json::Value>, AppError> {
    todo!()
}

/// POST /api/v1/orgs/:slug/invitations -- invite member
pub async fn handle_invite_member(
    Json(_req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    todo!()
}

/// DELETE /api/v1/orgs/:slug/members/:user_id
pub async fn handle_remove_member() -> Result<axum::http::StatusCode, AppError> {
    todo!()
}

/// PATCH /api/v1/orgs/:slug/members/:user_id/role
pub async fn handle_change_role(
    Json(_req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    todo!()
}
