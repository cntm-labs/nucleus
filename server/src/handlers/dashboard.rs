use std::sync::Arc;

use crate::core::error::AppError;
use crate::core::pagination::PaginationParams;
use axum::extract::{Path, Query, State};
use axum::Json;
use uuid::Uuid;

use crate::state::AppState;

// ---------------------------------------------------------------------------
// Projects
// ---------------------------------------------------------------------------

pub async fn handle_list_projects(
    State(state): State<Arc<AppState>>,
    query: Query<PaginationParams>,
) -> Result<Json<serde_json::Value>, AppError> {
    let ds = state.dashboard_state();
    crate::api::handlers::dashboard::handle_list_projects(State(ds), query).await
}

pub async fn handle_create_project(
    State(state): State<Arc<AppState>>,
    Json(req): Json<crate::api::handlers::dashboard::CreateProjectRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let ds = state.dashboard_state();
    crate::api::handlers::dashboard::handle_create_project(State(ds), Json(req)).await
}

pub async fn handle_get_project(
    State(state): State<Arc<AppState>>,
    path: Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let ds = state.dashboard_state();
    crate::api::handlers::dashboard::handle_get_project(State(ds), path).await
}

pub async fn handle_update_project(
    State(state): State<Arc<AppState>>,
    path: Path<Uuid>,
    Json(req): Json<crate::api::handlers::dashboard::UpdateProjectRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let ds = state.dashboard_state();
    crate::api::handlers::dashboard::handle_update_project(State(ds), path, Json(req)).await
}

// ---------------------------------------------------------------------------
// OAuth providers (stub pass-through)
// ---------------------------------------------------------------------------

pub async fn handle_list_providers() -> Result<Json<serde_json::Value>, AppError> {
    crate::api::handlers::dashboard::handle_list_providers().await
}

pub async fn handle_configure_provider(
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    crate::api::handlers::dashboard::handle_configure_provider(Json(req)).await
}

pub async fn handle_delete_provider() -> Result<axum::http::StatusCode, AppError> {
    crate::api::handlers::dashboard::handle_delete_provider().await
}

// ---------------------------------------------------------------------------
// API keys
// ---------------------------------------------------------------------------

pub async fn handle_list_api_keys(
    State(state): State<Arc<AppState>>,
    path: Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let ds = state.dashboard_state();
    crate::api::handlers::dashboard::handle_list_api_keys(State(ds), path).await
}

pub async fn handle_create_api_key(
    State(state): State<Arc<AppState>>,
    path: Path<Uuid>,
    Json(req): Json<crate::api::handlers::dashboard::CreateApiKeyRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let ds = state.dashboard_state();
    crate::api::handlers::dashboard::handle_create_api_key(State(ds), path, Json(req)).await
}

pub async fn handle_revoke_api_key(
    State(state): State<Arc<AppState>>,
    path: Path<(Uuid, Uuid)>,
) -> Result<axum::http::StatusCode, AppError> {
    let ds = state.dashboard_state();
    crate::api::handlers::dashboard::handle_revoke_api_key(State(ds), path).await
}

// ---------------------------------------------------------------------------
// Signing keys
// ---------------------------------------------------------------------------

pub async fn handle_list_signing_keys(
    State(state): State<Arc<AppState>>,
    path: Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let ds = state.dashboard_state();
    crate::api::handlers::dashboard::handle_list_signing_keys(State(ds), path).await
}

pub async fn handle_rotate_signing_key(
    State(state): State<Arc<AppState>>,
    path: Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let ds = state.dashboard_state();
    crate::api::handlers::dashboard::handle_rotate_signing_key(State(ds), path).await
}

// ---------------------------------------------------------------------------
// Templates (stub pass-through)
// ---------------------------------------------------------------------------

pub async fn handle_list_templates() -> Result<Json<serde_json::Value>, AppError> {
    crate::api::handlers::dashboard::handle_list_templates().await
}

pub async fn handle_update_template(
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    crate::api::handlers::dashboard::handle_update_template(Json(req)).await
}

pub async fn handle_reset_template() -> Result<Json<serde_json::Value>, AppError> {
    crate::api::handlers::dashboard::handle_reset_template().await
}

// ---------------------------------------------------------------------------
// JWT templates (stub pass-through)
// ---------------------------------------------------------------------------

pub async fn handle_list_jwt_templates() -> Result<Json<serde_json::Value>, AppError> {
    crate::api::handlers::dashboard::handle_list_jwt_templates().await
}

pub async fn handle_create_jwt_template(
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    crate::api::handlers::dashboard::handle_create_jwt_template(Json(req)).await
}

pub async fn handle_update_jwt_template(
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    crate::api::handlers::dashboard::handle_update_jwt_template(Json(req)).await
}

// ---------------------------------------------------------------------------
// Analytics (stub pass-through)
// ---------------------------------------------------------------------------

pub async fn handle_get_analytics() -> Result<Json<serde_json::Value>, AppError> {
    crate::api::handlers::dashboard::handle_get_analytics().await
}

// ---------------------------------------------------------------------------
// Billing (stub pass-through)
// ---------------------------------------------------------------------------

pub async fn handle_get_usage() -> Result<Json<serde_json::Value>, AppError> {
    crate::api::handlers::dashboard::handle_get_usage().await
}

pub async fn handle_get_subscription() -> Result<Json<serde_json::Value>, AppError> {
    crate::api::handlers::dashboard::handle_get_subscription().await
}

// ---------------------------------------------------------------------------
// Audit logs
// ---------------------------------------------------------------------------

pub async fn handle_list_audit_logs(
    State(state): State<Arc<AppState>>,
    path: Path<Uuid>,
    query: Query<PaginationParams>,
) -> Result<Json<serde_json::Value>, AppError> {
    let ds = state.dashboard_state();
    crate::api::handlers::dashboard::handle_list_audit_logs(State(ds), path, query).await
}

// ---------------------------------------------------------------------------
// Settings
// ---------------------------------------------------------------------------

pub async fn handle_get_settings(
    State(state): State<Arc<AppState>>,
    path: Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let ds = state.dashboard_state();
    crate::api::handlers::dashboard::handle_get_settings(State(ds), path).await
}

pub async fn handle_update_settings(
    State(state): State<Arc<AppState>>,
    path: Path<Uuid>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    let ds = state.dashboard_state();
    crate::api::handlers::dashboard::handle_update_settings(State(ds), path, Json(req)).await
}
