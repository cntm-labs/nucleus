use axum::{extract::Query, extract::State, http::StatusCode, Json};
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
    Json(req): Json<SignUpRequest>,
) -> Result<(StatusCode, Json<SignUpResponse>), AppError> {
    crate::auth::handlers::sign_up::handle_sign_up(State(state.auth_service.clone()), Json(req))
        .await
}

pub async fn handle_sign_in(
    State(state): State<Arc<AppState>>,
    Json(req): Json<SignInRequest>,
) -> Result<(StatusCode, Json<SignInResponse>), AppError> {
    crate::auth::handlers::sign_in::handle_sign_in(State(state.auth_service.clone()), Json(req))
        .await
}

#[derive(Debug, Deserialize)]
pub struct AuthRefreshRequest {
    pub session_id: String,
}

pub async fn handle_refresh(
    JwtAuth(claims): JwtAuth,
    State(state): State<Arc<AppState>>,
    Json(req): Json<AuthRefreshRequest>,
) -> Result<Json<RefreshResponse>, AppError> {
    let session_id: SessionId = req
        .session_id
        .parse()
        .map_err(|_| AppError::Auth(crate::core::error::AuthError::SessionExpired))?;
    let project_id: ProjectId = claims
        .aud
        .parse()
        .map_err(|_| AppError::Auth(crate::core::error::AuthError::TokenInvalid))?;

    let (jwt, expires_in) = state
        .auth_service
        .refresh_token(&state.session_service, &session_id, &project_id)
        .await?;

    Ok(Json(RefreshResponse { jwt, expires_in }))
}

#[derive(Debug, Deserialize)]
pub struct AuthSignOutRequest {
    pub session_id: String,
}

pub async fn handle_sign_out(
    JwtAuth(claims): JwtAuth,
    State(state): State<Arc<AppState>>,
    Json(req): Json<AuthSignOutRequest>,
) -> Result<StatusCode, AppError> {
    let session_id: SessionId = req
        .session_id
        .parse()
        .map_err(|_| AppError::Auth(crate::core::error::AuthError::SessionExpired))?;
    let user_id: UserId = claims
        .sub
        .parse()
        .map_err(|_| AppError::Auth(crate::core::error::AuthError::TokenInvalid))?;

    state
        .auth_service
        .sign_out(
            &state.session_service,
            &session_id,
            &user_id,
            Some(&claims.jti),
        )
        .await?;

    Ok(StatusCode::OK)
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
// Phase 3: OAuth (thin wrappers — need OAuthHandlerState not yet in AppState)
// ---------------------------------------------------------------------------

pub async fn handle_oauth_start(
    State(_state): State<Arc<AppState>>,
    Json(_req): Json<crate::auth::handlers::oauth::OAuthStartRequest>,
) -> Result<
    (
        StatusCode,
        Json<crate::auth::handlers::oauth::OAuthStartResponse>,
    ),
    AppError,
> {
    // OAuth start requires OAuthHandlerState which needs:
    // - HashMap<String, Arc<dyn OAuthProvider>> (configured providers)
    // - Arc<dyn OAuthStateStore> (Redis-backed state store)
    // These are not yet part of AppState. Once added, this handler will
    // construct OAuthHandlerState and delegate to
    // crate::auth::handlers::oauth::handle_oauth_start().
    Err(AppError::Auth(
        crate::core::error::AuthError::OAuthProviderError(
            "OAuth providers not yet configured in AppState".to_string(),
        ),
    ))
}

pub async fn handle_oauth_callback(
    State(_state): State<Arc<AppState>>,
    Query(_params): Query<crate::auth::handlers::oauth::OAuthCallbackParams>,
) -> Result<
    (
        StatusCode,
        Json<crate::auth::handlers::oauth::OAuthCallbackResponse>,
    ),
    AppError,
> {
    // OAuth callback requires OAuthHandlerState (same as handle_oauth_start).
    // Once providers and state_store are wired into AppState, this handler
    // will delegate to crate::auth::handlers::oauth::handle_oauth_callback().
    Err(AppError::Auth(
        crate::core::error::AuthError::OAuthProviderError(
            "OAuth providers not yet configured in AppState".to_string(),
        ),
    ))
}

// ---------------------------------------------------------------------------
// Phase 3: Magic Link (direct delegation — no state needed)
// ---------------------------------------------------------------------------

pub async fn handle_send_magic_link(
    State(state): State<Arc<AppState>>,
    Json(req): Json<crate::auth::handlers::magic_link::SendMagicLinkRequest>,
) -> Result<Json<crate::auth::handlers::magic_link::SendMagicLinkResponse>, AppError> {
    let magic_state = state.magic_link_state();
    crate::auth::handlers::magic_link::handle_send_magic_link(State(magic_state), Json(req)).await
}

pub async fn handle_verify_magic_link(
    State(state): State<Arc<AppState>>,
    Query(params): Query<crate::auth::handlers::magic_link::VerifyMagicLinkQuery>,
) -> Result<Json<crate::auth::handlers::magic_link::VerifyMagicLinkResponse>, AppError> {
    let magic_state = state.magic_link_state();
    crate::auth::handlers::magic_link::handle_verify_magic_link(State(magic_state), Query(params))
        .await
}

// ---------------------------------------------------------------------------
// Phase 3: OTP (direct delegation — no state needed)
// ---------------------------------------------------------------------------

pub async fn handle_send_otp(
    State(state): State<Arc<AppState>>,
    Json(req): Json<crate::auth::handlers::otp::SendOtpRequest>,
) -> Result<Json<crate::auth::handlers::otp::SendOtpResponse>, AppError> {
    let otp_state = state.otp_state();
    crate::auth::handlers::otp::handle_send_otp(State(otp_state), Json(req)).await
}

pub async fn handle_verify_otp(
    State(state): State<Arc<AppState>>,
    Json(req): Json<crate::auth::handlers::otp::VerifyOtpRequest>,
) -> Result<Json<crate::auth::handlers::otp::VerifyOtpResponse>, AppError> {
    let otp_state = state.otp_state();
    crate::auth::handlers::otp::handle_verify_otp(State(otp_state), Json(req)).await
}

// ---------------------------------------------------------------------------
// Phase 3: MFA (direct delegation — no state needed)
// ---------------------------------------------------------------------------

pub async fn handle_mfa_verify(
    State(state): State<Arc<AppState>>,
    Json(req): Json<crate::auth::handlers::mfa::MfaVerifyRequest>,
) -> Result<Json<crate::auth::handlers::mfa::MfaVerifyResponse>, AppError> {
    let mfa_state = state.mfa_state();
    crate::auth::handlers::mfa::handle_mfa_verify(State(mfa_state), Json(req)).await
}

// ---------------------------------------------------------------------------
// Phase 3: Passkeys (direct delegation — no state needed)
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

// ---------------------------------------------------------------------------
// Phase 3: Password Reset (direct delegation — no state needed)
// ---------------------------------------------------------------------------

pub async fn handle_request_reset(
    State(state): State<Arc<AppState>>,
    Json(req): Json<crate::auth::handlers::password_reset::RequestResetRequest>,
) -> Result<Json<crate::auth::handlers::password_reset::RequestResetResponse>, AppError> {
    let reset_state = state.password_reset_state();
    crate::auth::handlers::password_reset::handle_request_reset(State(reset_state), Json(req)).await
}

pub async fn handle_confirm_reset(
    State(state): State<Arc<AppState>>,
    Json(req): Json<crate::auth::handlers::password_reset::ConfirmResetRequest>,
) -> Result<Json<crate::auth::handlers::password_reset::ConfirmResetResponse>, AppError> {
    let reset_state = state.password_reset_state();
    crate::auth::handlers::password_reset::handle_confirm_reset(State(reset_state), Json(req)).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::error::AuthError;

    // Test the session_id and project_id parsing logic used by handlers.
    // Full handler tests require AppState with PgPool/Redis (integration test scope).

    #[test]
    fn valid_uuid_parses_as_session_id() {
        let valid = "550e8400-e29b-41d4-a716-446655440000";
        let result: Result<SessionId, _> = valid.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn invalid_uuid_fails_session_id_parse() {
        let invalid = "not-a-uuid";
        let result: Result<SessionId, _> = invalid.parse();
        assert!(result.is_err());
    }

    #[test]
    fn valid_uuid_parses_as_user_id() {
        let valid = "550e8400-e29b-41d4-a716-446655440000";
        let result: Result<UserId, _> = valid.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn invalid_uuid_fails_user_id_parse() {
        let invalid = "not-a-uuid";
        let result: Result<UserId, _> = invalid.parse();
        assert!(result.is_err());
    }

    #[test]
    fn invalid_session_id_maps_to_session_expired() {
        let bad_id = "not-a-uuid";
        let result: Result<SessionId, AppError> = bad_id
            .parse()
            .map_err(|_| AppError::Auth(AuthError::SessionExpired));
        assert!(matches!(
            result,
            Err(AppError::Auth(AuthError::SessionExpired))
        ));
    }

    #[test]
    fn invalid_user_id_maps_to_token_invalid() {
        let bad_id = "not-a-uuid";
        let result: Result<UserId, AppError> = bad_id
            .parse()
            .map_err(|_| AppError::Auth(AuthError::TokenInvalid));
        assert!(matches!(
            result,
            Err(AppError::Auth(AuthError::TokenInvalid))
        ));
    }

    #[test]
    fn request_types_deserialize_correctly() {
        let json = serde_json::json!({"session_id": "550e8400-e29b-41d4-a716-446655440000"});
        let req: AuthRefreshRequest = serde_json::from_value(json).unwrap();
        assert_eq!(req.session_id, "550e8400-e29b-41d4-a716-446655440000");

        let json = serde_json::json!({"session_id": "550e8400-e29b-41d4-a716-446655440000"});
        let req: AuthSignOutRequest = serde_json::from_value(json).unwrap();
        assert_eq!(req.session_id, "550e8400-e29b-41d4-a716-446655440000");
    }

    #[test]
    fn response_type_serializes_correctly() {
        let resp = AuthSignOutAllResponse { revoked_count: 5 };
        let json = serde_json::to_value(&resp).unwrap();
        assert_eq!(json["revoked_count"], 5);
    }
}
