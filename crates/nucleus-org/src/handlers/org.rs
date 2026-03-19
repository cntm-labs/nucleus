use axum::Json;
use nucleus_core::error::AppError;

/// GET /api/v1/orgs -- list user's orgs
pub async fn handle_list_orgs() -> Result<Json<serde_json::Value>, AppError> {
    todo!()
}

/// POST /api/v1/orgs -- create org
pub async fn handle_create_org(
    Json(_req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    todo!()
}

/// GET /api/v1/orgs/:slug -- org details
pub async fn handle_get_org() -> Result<Json<serde_json::Value>, AppError> {
    todo!()
}

/// PATCH /api/v1/orgs/:slug -- update org
pub async fn handle_update_org(
    Json(_req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    todo!()
}
