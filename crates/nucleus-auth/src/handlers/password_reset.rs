use std::sync::Arc;

use axum::extract::State;
use axum::Json;
use nucleus_core::error::{AppError, AuthError};
use nucleus_core::types::{ProjectId, UserId};
use nucleus_core::{crypto, validation};
use nucleus_db::repos::credential_repo::CredentialRepository;
use nucleus_db::repos::user_repo::UserRepository;
use nucleus_db::repos::verification_token_repo::VerificationTokenRepository;
use nucleus_session::session::SessionService;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::password_reset::PasswordResetService;

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct PasswordResetState {
    pub token_repo: Arc<dyn VerificationTokenRepository>,
    pub user_repo: Arc<dyn UserRepository>,
    pub credential_repo: Arc<dyn CredentialRepository>,
    pub session_service: Arc<SessionService>,
}

// ---------------------------------------------------------------------------
// Request Reset
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct RequestResetRequest {
    pub email: String,
    pub project_id: Uuid,
}

#[derive(Debug, Serialize)]
pub struct RequestResetResponse {
    pub message: String,
}

/// POST /auth/password/reset
/// Send password reset email. Always returns success (anti-enumeration).
pub async fn handle_request_reset(
    State(state): State<PasswordResetState>,
    Json(body): Json<RequestResetRequest>,
) -> Result<Json<RequestResetResponse>, AppError> {
    let project_id = ProjectId::from_uuid(body.project_id);

    // Look up user — always return success even if not found
    let user = state
        .user_repo
        .find_by_email(&project_id, &body.email)
        .await?;

    if let Some(user) = user {
        let generated = PasswordResetService::generate();

        state
            .token_repo
            .create(
                user.id.0,
                body.project_id,
                "password_reset",
                &generated.token_hash,
                None,
                generated.expires_at,
            )
            .await?;

        // TODO: Send email with reset link via email service
        tracing::debug!(
            email = %body.email,
            "Password reset token generated (email delivery not yet wired)"
        );
    }

    Ok(Json(RequestResetResponse {
        message: "If an account exists with this email, a password reset link has been sent."
            .to_string(),
    }))
}

// ---------------------------------------------------------------------------
// Confirm Reset
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct ConfirmResetRequest {
    pub token: String,
    pub new_password: String,
}

#[derive(Debug, Serialize)]
pub struct ConfirmResetResponse {
    pub message: String,
}

/// POST /auth/password/reset/confirm
/// Verify token and update password.
pub async fn handle_confirm_reset(
    State(state): State<PasswordResetState>,
    Json(body): Json<ConfirmResetRequest>,
) -> Result<Json<ConfirmResetResponse>, AppError> {
    // 1. Validate new password strength
    validation::validate_password(&body.new_password)?;

    // 2. Hash the provided token to look up in DB
    let token_hash = crypto::generate_token_hash(&body.token);

    // 3. Find stored token
    let stored = state
        .token_repo
        .find_by_hash(&token_hash, "password_reset")
        .await?
        .ok_or(AppError::Auth(AuthError::TokenInvalid))?;

    // 4. Verify: hash matches, not expired, not used
    PasswordResetService::verify_token(
        &body.token,
        &stored.token_hash,
        &stored.expires_at,
        &stored.used_at,
    )?;

    // 5. Mark token as used BEFORE updating password (prevent replay)
    state.token_repo.mark_used(stored.id).await?;

    // 6. Find user's password credential and update
    let user_id = UserId::from_uuid(stored.user_id);
    let project_id = ProjectId::from_uuid(stored.project_id);
    let credentials = state
        .credential_repo
        .find_by_user_and_type(&user_id, "password")
        .await?;

    let credential = credentials
        .first()
        .ok_or(AppError::Auth(AuthError::InvalidCredentials))?;

    let new_hash = crypto::hash_password(&body.new_password)?;
    state
        .credential_repo
        .update_secret(&credential.id, &new_hash)
        .await?;

    // 7. Revoke all sessions for security (force re-login)
    let _ = state.session_service.revoke_all_sessions(&user_id).await;

    tracing::info!(
        user_id = %user_id,
        project_id = %project_id,
        "Password reset completed, all sessions revoked"
    );

    Ok(Json(ConfirmResetResponse {
        message: "Password has been reset successfully.".to_string(),
    }))
}
