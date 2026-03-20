use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use nucleus_core::error::AppError;
use nucleus_core::pagination::PaginationParams;
use nucleus_core::types::{ProjectId, UserId};
use nucleus_identity::handlers::admin::{AdminUpdateUserRequest, CreateUserRequest};

use crate::state::AppState;

// ---------------------------------------------------------------------------
// Helpers: extract admin context
// ---------------------------------------------------------------------------

// TODO: Once admin API key middleware is fully wired, extract project_id from
// the validated API key. For now we use a placeholder so that handlers compile
// and the wiring is complete end-to-end.
fn placeholder_admin_project() -> ProjectId {
    ProjectId::new()
}

// ---------------------------------------------------------------------------
// Phase 4: Admin routes (thin wrappers that delegate to nucleus-identity)
// ---------------------------------------------------------------------------

/// GET /api/v1/admin/users
pub async fn handle_admin_list_users(
    State(state): State<Arc<AppState>>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<serde_json::Value>, AppError> {
    let project_id = placeholder_admin_project();
    let result = nucleus_identity::handlers::admin::list_users(
        &state.user_service,
        &project_id,
        &params,
    )
    .await?;
    Ok(Json(serde_json::to_value(result).unwrap_or_default()))
}

/// POST /api/v1/admin/users
pub async fn handle_admin_create_user(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateUserRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let project_id = placeholder_admin_project();
    let user = nucleus_identity::handlers::admin::create_user(
        &state.user_service,
        &project_id,
        req,
    )
    .await?;
    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(user).unwrap_or_default()),
    ))
}

/// GET /api/v1/admin/users/:id
pub async fn handle_admin_get_user(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<UserId>,
) -> Result<Json<serde_json::Value>, AppError> {
    let project_id = placeholder_admin_project();
    let user = nucleus_identity::handlers::admin::get_user(
        &state.user_service,
        &project_id,
        &user_id,
    )
    .await?;
    Ok(Json(serde_json::to_value(user).unwrap_or_default()))
}

/// PATCH /api/v1/admin/users/:id
pub async fn handle_admin_update_user(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<UserId>,
    Json(req): Json<AdminUpdateUserRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let project_id = placeholder_admin_project();
    let user = nucleus_identity::handlers::admin::update_user(
        &state.user_service,
        &project_id,
        &user_id,
        req,
    )
    .await?;
    Ok(Json(serde_json::to_value(user).unwrap_or_default()))
}

/// DELETE /api/v1/admin/users/:id
pub async fn handle_admin_delete_user(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<UserId>,
) -> Result<StatusCode, AppError> {
    let project_id = placeholder_admin_project();
    nucleus_identity::handlers::admin::delete_user(
        &state.user_service,
        &project_id,
        &user_id,
    )
    .await?;
    Ok(StatusCode::NO_CONTENT)
}

/// POST /api/v1/admin/users/:id/ban
pub async fn handle_admin_ban_user(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<UserId>,
) -> Result<StatusCode, AppError> {
    let project_id = placeholder_admin_project();
    nucleus_identity::handlers::admin::ban_user(
        &state.user_service,
        &project_id,
        &user_id,
    )
    .await?;
    Ok(StatusCode::NO_CONTENT)
}

/// POST /api/v1/admin/users/:id/unban
pub async fn handle_admin_unban_user(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<UserId>,
) -> Result<StatusCode, AppError> {
    let project_id = placeholder_admin_project();
    nucleus_identity::handlers::admin::unban_user(
        &state.user_service,
        &project_id,
        &user_id,
    )
    .await?;
    Ok(StatusCode::NO_CONTENT)
}
