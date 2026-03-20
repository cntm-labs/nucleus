use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use nucleus_core::error::AppError;
use nucleus_core::types::{ProjectId, SessionId, UserId};
use nucleus_identity::handlers::me::{UpdateProfileRequest, UserProfileResponse};

use crate::middleware::auth::JwtAuth;
use crate::state::AppState;

// ---------------------------------------------------------------------------
// Helpers: extract authenticated user context
// ---------------------------------------------------------------------------

/// Extract the project ID and user ID from validated JWT claims.
fn auth_from_jwt(jwt: &JwtAuth) -> (ProjectId, UserId) {
    let claims = &jwt.0;
    // `aud` carries the project ID; `sub` carries the user ID.
    let project_id = claims.aud.parse().unwrap_or_else(|_| ProjectId::new());
    let user_id = claims.sub.parse().unwrap_or_else(|_| UserId::new());
    (project_id, user_id)
}

// ---------------------------------------------------------------------------
// Phase 4: User profile routes (thin wrappers that delegate to nucleus-identity)
// ---------------------------------------------------------------------------

/// GET /api/v1/users/me
pub async fn handle_get_me(
    State(state): State<Arc<AppState>>,
    jwt: JwtAuth,
) -> Result<Json<UserProfileResponse>, AppError> {
    let (project_id, user_id) = auth_from_jwt(&jwt);
    let resp =
        nucleus_identity::handlers::me::get_me(&state.user_service, &project_id, &user_id).await?;
    Ok(Json(resp))
}

/// PATCH /api/v1/users/me
pub async fn handle_update_me(
    State(state): State<Arc<AppState>>,
    jwt: JwtAuth,
    Json(req): Json<UpdateProfileRequest>,
) -> Result<Json<UserProfileResponse>, AppError> {
    let (project_id, user_id) = auth_from_jwt(&jwt);
    let resp =
        nucleus_identity::handlers::me::update_me(&state.user_service, &project_id, &user_id, req)
            .await?;
    Ok(Json(resp))
}

/// DELETE /api/v1/users/me
pub async fn handle_delete_me(
    State(state): State<Arc<AppState>>,
    jwt: JwtAuth,
) -> Result<StatusCode, AppError> {
    let (project_id, user_id) = auth_from_jwt(&jwt);
    nucleus_identity::handlers::me::delete_me(&state.user_service, &project_id, &user_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// GET /api/v1/users/me/sessions
pub async fn handle_list_my_sessions(
    State(state): State<Arc<AppState>>,
    jwt: JwtAuth,
) -> Result<Json<serde_json::Value>, AppError> {
    let (_project_id, user_id) = auth_from_jwt(&jwt);
    let sessions = state.session_service.list_user_sessions(&user_id).await?;
    Ok(Json(serde_json::to_value(sessions).unwrap_or_default()))
}

/// DELETE /api/v1/users/me/sessions/:id
pub async fn handle_revoke_my_session(
    State(state): State<Arc<AppState>>,
    jwt: JwtAuth,
    Path(session_id): Path<SessionId>,
) -> Result<StatusCode, AppError> {
    let (_project_id, user_id) = auth_from_jwt(&jwt);
    state
        .session_service
        .revoke_session(&session_id, &user_id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
