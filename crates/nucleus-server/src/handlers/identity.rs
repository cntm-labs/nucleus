use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use nucleus_core::error::AppError;
use nucleus_core::types::{ProjectId, SessionId, UserId};
use nucleus_identity::handlers::me::{UpdateProfileRequest, UserProfileResponse};

use crate::state::AppState;

// ---------------------------------------------------------------------------
// Helpers: extract authenticated user context
// ---------------------------------------------------------------------------

// TODO: Once JWT auth middleware is fully wired, extract project_id and user_id
// from validated JWT claims in request extensions. For now we use placeholder
// values so that handlers compile and the wiring is complete end-to-end.
fn placeholder_auth() -> (ProjectId, UserId) {
    // This will be replaced by middleware-provided claims
    (ProjectId::new(), UserId::new())
}

// ---------------------------------------------------------------------------
// Phase 4: User profile routes (thin wrappers that delegate to nucleus-identity)
// ---------------------------------------------------------------------------

/// GET /api/v1/users/me
pub async fn handle_get_me(
    State(state): State<Arc<AppState>>,
) -> Result<Json<UserProfileResponse>, AppError> {
    let (project_id, user_id) = placeholder_auth();
    let resp = nucleus_identity::handlers::me::get_me(
        &state.user_service,
        &project_id,
        &user_id,
    )
    .await?;
    Ok(Json(resp))
}

/// PATCH /api/v1/users/me
pub async fn handle_update_me(
    State(state): State<Arc<AppState>>,
    Json(req): Json<UpdateProfileRequest>,
) -> Result<Json<UserProfileResponse>, AppError> {
    let (project_id, user_id) = placeholder_auth();
    let resp = nucleus_identity::handlers::me::update_me(
        &state.user_service,
        &project_id,
        &user_id,
        req,
    )
    .await?;
    Ok(Json(resp))
}

/// DELETE /api/v1/users/me
pub async fn handle_delete_me(
    State(state): State<Arc<AppState>>,
) -> Result<StatusCode, AppError> {
    let (project_id, user_id) = placeholder_auth();
    nucleus_identity::handlers::me::delete_me(
        &state.user_service,
        &project_id,
        &user_id,
    )
    .await?;
    Ok(StatusCode::NO_CONTENT)
}

/// GET /api/v1/users/me/sessions
pub async fn handle_list_my_sessions(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, AppError> {
    let (_project_id, user_id) = placeholder_auth();
    let sessions = state.session_service.list_user_sessions(&user_id).await?;
    Ok(Json(serde_json::to_value(sessions).unwrap_or_default()))
}

/// DELETE /api/v1/users/me/sessions/:id
pub async fn handle_revoke_my_session(
    State(state): State<Arc<AppState>>,
    Path(session_id): Path<SessionId>,
) -> Result<StatusCode, AppError> {
    let (_project_id, user_id) = placeholder_auth();
    state
        .session_service
        .revoke_session(&session_id, &user_id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
