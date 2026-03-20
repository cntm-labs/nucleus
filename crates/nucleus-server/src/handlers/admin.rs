use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use nucleus_core::error::AppError;
use nucleus_core::pagination::PaginationParams;
use nucleus_core::types::UserId;
use nucleus_identity::handlers::admin::{AdminUpdateUserRequest, CreateUserRequest};

use crate::middleware::auth::ApiKeyAuth;
use crate::state::AppState;

// ---------------------------------------------------------------------------
// Phase 4: Admin routes (thin wrappers that delegate to nucleus-identity)
// Require a valid API key (sk_ prefix) via the ApiKeyAuth extractor.
// ---------------------------------------------------------------------------

/// Verify the API key is a secret key with admin scope.
fn require_admin(api_key: &ApiKeyAuth) -> Result<(), AppError> {
    if api_key.key_type != "secret" {
        return Err(AppError::Auth(nucleus_core::error::AuthError::TokenInvalid));
    }
    if !api_key.scopes.is_empty() && !api_key.scopes.iter().any(|s| s == "admin" || s == "*") {
        return Err(AppError::Auth(nucleus_core::error::AuthError::TokenInvalid));
    }
    Ok(())
}

/// GET /api/v1/admin/users
pub async fn handle_admin_list_users(
    State(state): State<Arc<AppState>>,
    api_key: ApiKeyAuth,
    Query(params): Query<PaginationParams>,
) -> Result<Json<serde_json::Value>, AppError> {
    require_admin(&api_key)?;
    let result = nucleus_identity::handlers::admin::list_users(
        &state.user_service,
        &api_key.project_id,
        &params,
    )
    .await?;
    Ok(Json(serde_json::to_value(result).unwrap_or_default()))
}

/// POST /api/v1/admin/users
pub async fn handle_admin_create_user(
    State(state): State<Arc<AppState>>,
    api_key: ApiKeyAuth,
    Json(req): Json<CreateUserRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let user = nucleus_identity::handlers::admin::create_user(
        &state.user_service,
        &api_key.project_id,
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
    api_key: ApiKeyAuth,
    Path(user_id): Path<UserId>,
) -> Result<Json<serde_json::Value>, AppError> {
    let user = nucleus_identity::handlers::admin::get_user(
        &state.user_service,
        &api_key.project_id,
        &user_id,
    )
    .await?;
    Ok(Json(serde_json::to_value(user).unwrap_or_default()))
}

/// PATCH /api/v1/admin/users/:id
pub async fn handle_admin_update_user(
    State(state): State<Arc<AppState>>,
    api_key: ApiKeyAuth,
    Path(user_id): Path<UserId>,
    Json(req): Json<AdminUpdateUserRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let user = nucleus_identity::handlers::admin::update_user(
        &state.user_service,
        &api_key.project_id,
        &user_id,
        req,
    )
    .await?;
    Ok(Json(serde_json::to_value(user).unwrap_or_default()))
}

/// DELETE /api/v1/admin/users/:id
pub async fn handle_admin_delete_user(
    State(state): State<Arc<AppState>>,
    api_key: ApiKeyAuth,
    Path(user_id): Path<UserId>,
) -> Result<StatusCode, AppError> {
    nucleus_identity::handlers::admin::delete_user(
        &state.user_service,
        &api_key.project_id,
        &user_id,
    )
    .await?;
    Ok(StatusCode::NO_CONTENT)
}

/// POST /api/v1/admin/users/:id/ban
pub async fn handle_admin_ban_user(
    State(state): State<Arc<AppState>>,
    api_key: ApiKeyAuth,
    Path(user_id): Path<UserId>,
) -> Result<StatusCode, AppError> {
    nucleus_identity::handlers::admin::ban_user(&state.user_service, &api_key.project_id, &user_id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

/// POST /api/v1/admin/users/:id/unban
pub async fn handle_admin_unban_user(
    State(state): State<Arc<AppState>>,
    api_key: ApiKeyAuth,
    Path(user_id): Path<UserId>,
) -> Result<StatusCode, AppError> {
    nucleus_identity::handlers::admin::unban_user(
        &state.user_service,
        &api_key.project_id,
        &user_id,
    )
    .await?;
    Ok(StatusCode::NO_CONTENT)
}
