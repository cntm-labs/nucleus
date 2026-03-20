use axum::{extract::Query, extract::State, http::StatusCode, Json};
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

// ---------------------------------------------------------------------------
// Phase 3: OAuth (thin wrappers — need OAuthHandlerState not yet in AppState)
// ---------------------------------------------------------------------------

pub async fn handle_oauth_start(
    State(_state): State<Arc<AppState>>,
    Json(_req): Json<nucleus_auth::handlers::oauth::OAuthStartRequest>,
) -> Result<(StatusCode, Json<nucleus_auth::handlers::oauth::OAuthStartResponse>), AppError> {
    // OAuth start requires OAuthHandlerState which needs:
    // - HashMap<String, Arc<dyn OAuthProvider>> (configured providers)
    // - Arc<dyn OAuthStateStore> (Redis-backed state store)
    // These are not yet part of AppState. Once added, this handler will
    // construct OAuthHandlerState and delegate to
    // nucleus_auth::handlers::oauth::handle_oauth_start().
    Err(AppError::Auth(
        nucleus_core::error::AuthError::OAuthProviderError(
            "OAuth providers not yet configured in AppState".to_string(),
        ),
    ))
}

pub async fn handle_oauth_callback(
    State(_state): State<Arc<AppState>>,
    Query(_params): Query<nucleus_auth::handlers::oauth::OAuthCallbackParams>,
) -> Result<(StatusCode, Json<nucleus_auth::handlers::oauth::OAuthCallbackResponse>), AppError> {
    // OAuth callback requires OAuthHandlerState (same as handle_oauth_start).
    // Once providers and state_store are wired into AppState, this handler
    // will delegate to nucleus_auth::handlers::oauth::handle_oauth_callback().
    Err(AppError::Auth(
        nucleus_core::error::AuthError::OAuthProviderError(
            "OAuth providers not yet configured in AppState".to_string(),
        ),
    ))
}

// ---------------------------------------------------------------------------
// Phase 3: Magic Link (direct delegation — no state needed)
// ---------------------------------------------------------------------------

pub async fn handle_send_magic_link(
    Json(req): Json<nucleus_auth::handlers::magic_link::SendMagicLinkRequest>,
) -> Result<Json<nucleus_auth::handlers::magic_link::SendMagicLinkResponse>, AppError> {
    nucleus_auth::handlers::magic_link::handle_send_magic_link(Json(req)).await
}

pub async fn handle_verify_magic_link(
    Query(params): Query<nucleus_auth::handlers::magic_link::VerifyMagicLinkQuery>,
) -> Result<Json<nucleus_auth::handlers::magic_link::VerifyMagicLinkResponse>, AppError> {
    nucleus_auth::handlers::magic_link::handle_verify_magic_link(Query(params)).await
}

// ---------------------------------------------------------------------------
// Phase 3: OTP (direct delegation — no state needed)
// ---------------------------------------------------------------------------

pub async fn handle_send_otp(
    Json(req): Json<nucleus_auth::handlers::otp::SendOtpRequest>,
) -> Result<Json<nucleus_auth::handlers::otp::SendOtpResponse>, AppError> {
    nucleus_auth::handlers::otp::handle_send_otp(Json(req)).await
}

pub async fn handle_verify_otp(
    Json(req): Json<nucleus_auth::handlers::otp::VerifyOtpRequest>,
) -> Result<Json<nucleus_auth::handlers::otp::VerifyOtpResponse>, AppError> {
    nucleus_auth::handlers::otp::handle_verify_otp(Json(req)).await
}

// ---------------------------------------------------------------------------
// Phase 3: MFA (direct delegation — no state needed)
// ---------------------------------------------------------------------------

pub async fn handle_mfa_verify(
    Json(req): Json<nucleus_auth::handlers::mfa::MfaVerifyRequest>,
) -> Result<Json<nucleus_auth::handlers::mfa::MfaVerifyResponse>, AppError> {
    nucleus_auth::handlers::mfa::handle_mfa_verify(Json(req)).await
}

// ---------------------------------------------------------------------------
// Phase 3: Passkeys (direct delegation — no state needed)
// ---------------------------------------------------------------------------

pub async fn handle_passkey_register_begin(
    Json(req): Json<nucleus_auth::handlers::passkey::PasskeyRegisterBeginRequest>,
) -> Result<Json<nucleus_auth::handlers::passkey::PasskeyRegisterBeginResponse>, AppError> {
    nucleus_auth::handlers::passkey::handle_passkey_register_begin(Json(req)).await
}

pub async fn handle_passkey_register_finish(
    Json(req): Json<nucleus_auth::handlers::passkey::PasskeyRegisterFinishRequest>,
) -> Result<Json<nucleus_auth::handlers::passkey::PasskeyRegisterFinishResponse>, AppError> {
    nucleus_auth::handlers::passkey::handle_passkey_register_finish(Json(req)).await
}

pub async fn handle_passkey_auth_begin(
    Json(req): Json<nucleus_auth::handlers::passkey::PasskeyAuthBeginRequest>,
) -> Result<Json<nucleus_auth::handlers::passkey::PasskeyAuthBeginResponse>, AppError> {
    nucleus_auth::handlers::passkey::handle_passkey_auth_begin(Json(req)).await
}

pub async fn handle_passkey_auth_finish(
    Json(req): Json<nucleus_auth::handlers::passkey::PasskeyAuthFinishRequest>,
) -> Result<Json<nucleus_auth::handlers::passkey::PasskeyAuthFinishResponse>, AppError> {
    nucleus_auth::handlers::passkey::handle_passkey_auth_finish(Json(req)).await
}

// ---------------------------------------------------------------------------
// Phase 3: Password Reset (direct delegation — no state needed)
// ---------------------------------------------------------------------------

pub async fn handle_request_reset(
    Json(req): Json<nucleus_auth::handlers::password_reset::RequestResetRequest>,
) -> Result<Json<nucleus_auth::handlers::password_reset::RequestResetResponse>, AppError> {
    nucleus_auth::handlers::password_reset::handle_request_reset(Json(req)).await
}

pub async fn handle_confirm_reset(
    Json(req): Json<nucleus_auth::handlers::password_reset::ConfirmResetRequest>,
) -> Result<Json<nucleus_auth::handlers::password_reset::ConfirmResetResponse>, AppError> {
    nucleus_auth::handlers::password_reset::handle_confirm_reset(Json(req)).await
}
