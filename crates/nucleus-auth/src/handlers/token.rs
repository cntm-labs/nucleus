use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::service::AuthService;
use nucleus_core::error::AppError;
use nucleus_core::types::{ProjectId, SessionId, UserId};
use nucleus_session::session::SessionService;

/// Shared state required by token handlers.
pub struct TokenState {
    pub auth_service: Arc<AuthService>,
    pub session_service: Arc<SessionService>,
}

#[derive(Debug, Deserialize)]
pub struct RefreshRequest {
    pub session_id: String,
}

#[derive(Debug, Serialize)]
pub struct RefreshResponse {
    pub jwt: String,
    pub expires_in: i64,
}

/// POST /api/v1/auth/refresh
///
/// Accepts a session_id and returns a fresh JWT if the session is still valid.
pub async fn handle_refresh(
    State(state): State<Arc<TokenState>>,
    Json(req): Json<RefreshRequest>,
) -> Result<Json<RefreshResponse>, AppError> {
    let session_id: SessionId = req
        .session_id
        .parse()
        .map_err(|_| AppError::Auth(nucleus_core::error::AuthError::SessionExpired))?;

    // TODO: project_id will come from middleware (API key extraction)
    let project_id = ProjectId::new();

    let (jwt, expires_in) = state
        .auth_service
        .refresh_token(&state.session_service, &session_id, &project_id)
        .await?;

    Ok(Json(RefreshResponse { jwt, expires_in }))
}

/// POST /api/v1/auth/sign-out
///
/// Revokes the current session. In production, session_id and user_id come
/// from the authenticated JWT claims; for now they are accepted as JSON body.
#[derive(Debug, Deserialize)]
pub struct SignOutRequest {
    pub session_id: String,
    pub user_id: String,
}

pub async fn handle_sign_out(
    State(state): State<Arc<TokenState>>,
    Json(req): Json<SignOutRequest>,
) -> Result<StatusCode, AppError> {
    let session_id: SessionId = req
        .session_id
        .parse()
        .map_err(|_| AppError::Auth(nucleus_core::error::AuthError::SessionExpired))?;
    let user_id: UserId = req
        .user_id
        .parse()
        .map_err(|_| AppError::Auth(nucleus_core::error::AuthError::TokenInvalid))?;

    state
        .auth_service
        .sign_out(&state.session_service, &session_id, &user_id, None)
        .await?;

    Ok(StatusCode::OK)
}

/// POST /api/v1/auth/sign-out-all
///
/// Revokes all sessions for the user. In production, user_id comes from JWT
/// claims; for now it is accepted as JSON body.
#[derive(Debug, Deserialize)]
pub struct SignOutAllRequest {
    pub user_id: String,
}

#[derive(Debug, Serialize)]
pub struct SignOutAllResponse {
    pub revoked_count: u64,
}

pub async fn handle_sign_out_all(
    State(state): State<Arc<TokenState>>,
    Json(req): Json<SignOutAllRequest>,
) -> Result<Json<SignOutAllResponse>, AppError> {
    let user_id: UserId = req
        .user_id
        .parse()
        .map_err(|_| AppError::Auth(nucleus_core::error::AuthError::TokenInvalid))?;

    let revoked_count = state
        .auth_service
        .sign_out_all(&state.session_service, &user_id)
        .await?;

    Ok(Json(SignOutAllResponse { revoked_count }))
}
