use axum::{extract::State, http::StatusCode, Json};
use std::sync::Arc;

use nucleus_auth::handlers::sign_in::{SignInRequest, SignInResponse};
use nucleus_auth::handlers::sign_up::{SignUpRequest, SignUpResponse};
use nucleus_auth::handlers::token::{
    RefreshRequest, RefreshResponse, SignOutAllRequest, SignOutAllResponse, SignOutRequest,
};
use nucleus_core::error::AppError;

use crate::state::AppState;

// Re-use the response types from nucleus-auth but delegate to the service layer directly.

pub async fn handle_sign_up(
    State(state): State<Arc<AppState>>,
    Json(req): Json<SignUpRequest>,
) -> Result<(StatusCode, Json<SignUpResponse>), AppError> {
    nucleus_auth::handlers::sign_up::handle_sign_up(
        State(state.auth_service.clone()),
        Json(req),
    )
    .await
}

pub async fn handle_sign_in(
    State(state): State<Arc<AppState>>,
    Json(req): Json<SignInRequest>,
) -> Result<(StatusCode, Json<SignInResponse>), AppError> {
    nucleus_auth::handlers::sign_in::handle_sign_in(
        State(state.auth_service.clone()),
        Json(req),
    )
    .await
}

pub async fn handle_refresh(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RefreshRequest>,
) -> Result<Json<RefreshResponse>, AppError> {
    let token_state = Arc::new(nucleus_auth::handlers::token::TokenState {
        auth_service: state.auth_service.clone(),
        session_service: state.session_service.clone(),
    });
    nucleus_auth::handlers::token::handle_refresh(State(token_state), Json(req)).await
}

pub async fn handle_sign_out(
    State(state): State<Arc<AppState>>,
    Json(req): Json<SignOutRequest>,
) -> Result<StatusCode, AppError> {
    let token_state = Arc::new(nucleus_auth::handlers::token::TokenState {
        auth_service: state.auth_service.clone(),
        session_service: state.session_service.clone(),
    });
    nucleus_auth::handlers::token::handle_sign_out(State(token_state), Json(req)).await
}

pub async fn handle_sign_out_all(
    State(state): State<Arc<AppState>>,
    Json(req): Json<SignOutAllRequest>,
) -> Result<Json<SignOutAllResponse>, AppError> {
    let token_state = Arc::new(nucleus_auth::handlers::token::TokenState {
        auth_service: state.auth_service.clone(),
        session_service: state.session_service.clone(),
    });
    nucleus_auth::handlers::token::handle_sign_out_all(State(token_state), Json(req)).await
}
