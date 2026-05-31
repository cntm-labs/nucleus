use axum::{
    extract::Extension, extract::Query, extract::State, http::HeaderMap, http::StatusCode, Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::auth::handlers::sign_in::{SignInRequest, SignInResponse};
use crate::auth::handlers::sign_up::{SignUpRequest, SignUpResponse};
use crate::auth::handlers::token::RefreshResponse;
use crate::core::error::AppError;
use crate::core::types::{ProjectId, SessionId, UserId};

use crate::middleware::auth::JwtAuth;
use crate::state::AppState;

// Re-use the response types from nucleus-auth but delegate to the service layer directly.

pub async fn handle_sign_up(
    State(state): State<Arc<AppState>>,
    Extension(project_id): Extension<ProjectId>,
    Json(req): Json<SignUpRequest>,
) -> Result<(StatusCode, Json<SignUpResponse>), AppError> {
    crate::auth::handlers::sign_up::handle_sign_up(
        State(state.auth_service.clone()),
        Extension(project_id),
        Json(req),
    )
    .await
}

pub async fn handle_sign_in(
    State(state): State<Arc<AppState>>,
    Extension(project_id): Extension<ProjectId>,
    headers: HeaderMap,
    Json(req): Json<SignInRequest>,
) -> Result<(StatusCode, Json<SignInResponse>), AppError> {
    crate::auth::handlers::sign_in::handle_sign_in(
        State(state.auth_service.clone()),
        Extension(project_id),
        headers,
        Json(req),
    )
    .await
}

#[derive(Debug, Deserialize)]
pub struct AuthRefreshRequest {
    pub session_id: String,
}

pub async fn handle_refresh(
    State(state): State<Arc<AppState>>,
    Extension(project_id): Extension<ProjectId>,
    Json(req): Json<AuthRefreshRequest>,
) -> Result<Json<RefreshResponse>, AppError> {
    let session_id = SessionId::from_uuid(
        req.session_id
            .parse()
            .map_err(|_| AppError::Auth(crate::core::error::AuthError::SessionInvalid))?,
    );

    let (jwt, lifetime) = state
        .auth_service
        .refresh_token(&state.session_service, &session_id, &project_id)
        .await?;

    Ok(Json(RefreshResponse {
        jwt,
        expires_in: lifetime,
    }))
}

pub async fn handle_sign_out(
    JwtAuth(claims): JwtAuth,
    State(state): State<Arc<AppState>>,
) -> Result<StatusCode, AppError> {
    let session_id = SessionId::from_uuid(
        claims
            .sid
            .parse()
            .map_err(|_| AppError::Auth(crate::core::error::AuthError::TokenInvalid))?,
    );

    let user_id = UserId::from_uuid(
        claims
            .sub
            .parse()
            .map_err(|_| AppError::Auth(crate::core::error::AuthError::TokenInvalid))?,
    );

    let jti = claims.jti.as_str();

    state
        .auth_service
        .sign_out(&state.session_service, &session_id, &user_id, Some(jti))
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, Serialize)]
pub struct AuthSignOutAllResponse {
    pub revoked_count: u64,
}

pub async fn handle_sign_out_all(
    JwtAuth(claims): JwtAuth,
    State(state): State<Arc<AppState>>,
) -> Result<Json<AuthSignOutAllResponse>, AppError> {
    let user_id: UserId = claims
        .sub
        .parse()
        .map_err(|_| AppError::Auth(crate::core::error::AuthError::TokenInvalid))?;

    let revoked_count = state
        .auth_service
        .sign_out_all(&state.session_service, &user_id)
        .await?;

    Ok(Json(AuthSignOutAllResponse { revoked_count }))
}

// ---------------------------------------------------------------------------
// Phase 3: OAuth
// ---------------------------------------------------------------------------

pub async fn handle_oauth_start(
    State(state): State<Arc<AppState>>,
    Extension(project_id): Extension<ProjectId>,
    Json(req): Json<crate::auth::handlers::oauth::OAuthStartRequest>,
) -> Result<
    (
        StatusCode,
        Json<crate::auth::handlers::oauth::OAuthStartResponse>,
    ),
    AppError,
> {
    let oauth_state = state.oauth_handler_state();
    crate::auth::handlers::oauth::handle_oauth_start(State(oauth_state), project_id, Json(req))
        .await
}

pub async fn handle_oauth_callback(
    State(state): State<Arc<AppState>>,
    query: Query<crate::auth::handlers::oauth::OAuthCallbackParams>,
) -> Result<
    (
        StatusCode,
        Json<crate::auth::handlers::oauth::OAuthCallbackResponse>,
    ),
    AppError,
> {
    let oauth_state = state.oauth_handler_state();
    crate::auth::handlers::oauth::handle_oauth_callback(State(oauth_state), query).await
}

// ---------------------------------------------------------------------------
// Phase 3: Magic Links (direct delegation)
// ---------------------------------------------------------------------------

pub async fn handle_magic_link_send(
    State(state): State<Arc<AppState>>,
    Json(req): Json<crate::auth::handlers::magic_link::SendMagicLinkRequest>,
) -> Result<Json<crate::auth::handlers::magic_link::SendMagicLinkResponse>, AppError> {
    let ml_state = state.magic_link_state();
    crate::auth::handlers::magic_link::handle_send_magic_link(State(ml_state), Json(req)).await
}

pub async fn handle_magic_link_verify(
    State(state): State<Arc<AppState>>,
    query: Query<crate::auth::handlers::magic_link::VerifyMagicLinkQuery>,
) -> Result<Json<crate::auth::handlers::magic_link::VerifyMagicLinkResponse>, AppError> {
    let ml_state = state.magic_link_state();
    crate::auth::handlers::magic_link::handle_verify_magic_link(State(ml_state), query).await
}

// ---------------------------------------------------------------------------
// Phase 3: Password Reset (direct delegation)
// ---------------------------------------------------------------------------

pub async fn handle_password_reset_send(
    State(state): State<Arc<AppState>>,
    Json(req): Json<crate::auth::handlers::password_reset::RequestResetRequest>,
) -> Result<Json<crate::auth::handlers::password_reset::RequestResetResponse>, AppError> {
    let pr_state = state.password_reset_state();
    crate::auth::handlers::password_reset::handle_request_reset(State(pr_state), Json(req)).await
}

pub async fn handle_password_reset_verify(
    State(state): State<Arc<AppState>>,
    Json(req): Json<crate::auth::handlers::password_reset::ConfirmResetRequest>,
) -> Result<Json<crate::auth::handlers::password_reset::ConfirmResetResponse>, AppError> {
    let pr_state = state.password_reset_state();
    crate::auth::handlers::password_reset::handle_confirm_reset(State(pr_state), Json(req)).await
}

// ---------------------------------------------------------------------------
// Phase 3: OTP (direct delegation)
// ---------------------------------------------------------------------------

pub async fn handle_send_otp(
    State(state): State<Arc<AppState>>,
    Extension(project_id): Extension<ProjectId>,
    Json(req): Json<crate::auth::handlers::otp::SendOtpRequest>,
) -> Result<Json<crate::auth::handlers::otp::SendOtpResponse>, AppError> {
    let otp_state = state.otp_state();
    crate::auth::handlers::otp::handle_send_otp(State(otp_state), Extension(project_id), Json(req))
        .await
}

pub async fn handle_verify_otp(
    State(state): State<Arc<AppState>>,
    Extension(project_id): Extension<ProjectId>,
    Json(req): Json<crate::auth::handlers::otp::VerifyOtpRequest>,
) -> Result<Json<crate::auth::handlers::otp::VerifyOtpResponse>, AppError> {
    let otp_state = state.otp_state();
    crate::auth::handlers::otp::handle_verify_otp(
        State(otp_state),
        Extension(project_id),
        Json(req),
    )
    .await
}

// ---------------------------------------------------------------------------
// Phase 3: MFA (direct delegation)
// ---------------------------------------------------------------------------

pub async fn handle_mfa_totp_enroll(
    State(state): State<Arc<AppState>>,
    JwtAuth(claims): JwtAuth,
    Json(req): Json<crate::auth::handlers::mfa::TotpEnrollRequest>,
) -> Result<Json<crate::auth::handlers::mfa::TotpEnrollResponse>, AppError> {
    let user_id = UserId::from_uuid(
        claims
            .sub
            .parse()
            .map_err(|_| AppError::Auth(crate::core::error::AuthError::TokenInvalid))?,
    );
    let mfa_state = state.mfa_state();
    crate::auth::handlers::mfa::handle_mfa_totp_enroll(
        State(mfa_state),
        user_id,
        claims.email.unwrap_or_default(),
        Json(req),
    )
    .await
}

pub async fn handle_mfa_totp_verify(
    State(state): State<Arc<AppState>>,
    JwtAuth(claims): JwtAuth,
    Json(req): Json<crate::auth::handlers::mfa::TotpVerifyRequest>,
) -> Result<Json<crate::auth::handlers::mfa::TotpVerifyResponse>, AppError> {
    let user_id = UserId::from_uuid(
        claims
            .sub
            .parse()
            .map_err(|_| AppError::Auth(crate::core::error::AuthError::TokenInvalid))?,
    );
    let mfa_state = state.mfa_state();
    crate::auth::handlers::mfa::handle_mfa_totp_verify(State(mfa_state), user_id, Json(req)).await
}

pub async fn handle_mfa_verify(
    State(state): State<Arc<AppState>>,
    Json(req): Json<crate::auth::handlers::mfa::MfaVerifyRequest>,
) -> Result<Json<crate::auth::handlers::mfa::MfaVerifyResponse>, AppError> {
    let mfa_state = state.mfa_state();
    crate::auth::handlers::mfa::handle_mfa_verify(State(mfa_state), Json(req)).await
}

// ---------------------------------------------------------------------------
// Phase 3: Passkeys (stub placeholders)
// ---------------------------------------------------------------------------

pub async fn handle_passkey_register_begin(
    State(state): State<Arc<AppState>>,
    Json(req): Json<crate::auth::handlers::passkey::PasskeyRegisterBeginRequest>,
) -> Result<Json<crate::auth::handlers::passkey::PasskeyRegisterBeginResponse>, AppError> {
    crate::auth::handlers::passkey::handle_passkey_register_begin(
        &state.rp_name,
        &state.rp_id,
        Json(req),
    )
    .await
}

pub async fn handle_passkey_register_finish(
    Json(req): Json<crate::auth::handlers::passkey::PasskeyRegisterFinishRequest>,
) -> Result<Json<crate::auth::handlers::passkey::PasskeyRegisterFinishResponse>, AppError> {
    crate::auth::handlers::passkey::handle_passkey_register_finish(Json(req)).await
}

pub async fn handle_passkey_auth_begin(
    Json(req): Json<crate::auth::handlers::passkey::PasskeyAuthBeginRequest>,
) -> Result<Json<crate::auth::handlers::passkey::PasskeyAuthBeginResponse>, AppError> {
    crate::auth::handlers::passkey::handle_passkey_auth_begin(Json(req)).await
}

pub async fn handle_passkey_auth_finish(
    Json(req): Json<crate::auth::handlers::passkey::PasskeyAuthFinishRequest>,
) -> Result<Json<crate::auth::handlers::passkey::PasskeyAuthFinishResponse>, AppError> {
    crate::auth::handlers::passkey::handle_passkey_auth_finish(Json(req)).await
}
