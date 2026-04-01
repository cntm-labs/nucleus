use std::sync::Arc;

use axum::extract::{Query, State};
use axum::Json;
use nucleus_core::error::{AppError, AuthError, UserError};
use nucleus_core::notification::NotificationService;
use nucleus_core::types::ProjectId;
use nucleus_db::repos::user_repo::UserRepository;
use nucleus_db::repos::verification_token_repo::VerificationTokenRepository;
use nucleus_session::session::{DeviceInfo, SessionService};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::magic_link::MagicLinkService;
use crate::service::AuthService;

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct MagicLinkState {
    pub token_repo: Arc<dyn VerificationTokenRepository>,
    pub user_repo: Arc<dyn UserRepository>,
    pub session_service: Arc<SessionService>,
    pub auth_service: Arc<AuthService>,
    pub notification_service: Arc<dyn NotificationService>,
    pub base_url: String,
}

// ---------------------------------------------------------------------------
// Send Magic Link
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct SendMagicLinkRequest {
    pub email: String,
    pub redirect_url: String,
    pub project_id: Uuid,
}

#[derive(Debug, Serialize)]
pub struct SendMagicLinkResponse {
    pub message: String,
}

/// POST /auth/sign-in/magic-link
/// Generates a magic link and "sends" it (in production, via email service).
/// Always returns 200 (anti-enumeration: don't reveal if email exists).
pub async fn handle_send_magic_link(
    State(state): State<MagicLinkState>,
    Json(body): Json<SendMagicLinkRequest>,
) -> Result<Json<SendMagicLinkResponse>, AppError> {
    let project_id = ProjectId::from_uuid(body.project_id);

    // Look up user — but always return success (anti-enumeration)
    let user = state
        .user_repo
        .find_by_email(&project_id, &body.email)
        .await?;

    if let Some(user) = user {
        let generated = MagicLinkService::generate();

        state
            .token_repo
            .create(
                user.id.0,
                body.project_id,
                "magic_link",
                &generated.token_hash,
                Some(&body.redirect_url),
                generated.expires_at,
            )
            .await?;

        let magic_url = MagicLinkService::build_url(
            &state.base_url,
            &generated.token,
            &body.redirect_url,
            &[],
        )?;
        let _ = state
            .notification_service
            .send_email(
                &body.email,
                "Sign in to Nucleus",
                &format!(
                    "<p>Click <a href=\"{}\">here</a> to sign in. This link expires in 15 minutes.</p>",
                    magic_url
                ),
                &format!("Sign in: {} (expires in 15 minutes)", magic_url),
            )
            .await;
    }

    Ok(Json(SendMagicLinkResponse {
        message: "If an account exists with this email, a magic link has been sent.".to_string(),
    }))
}

// ---------------------------------------------------------------------------
// Verify Magic Link
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct VerifyMagicLinkQuery {
    pub token: String,
    pub redirect_url: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct VerifyMagicLinkResponse {
    pub user: serde_json::Value,
    pub jwt: String,
    pub session_token: String,
}

/// GET /auth/magic-link/verify?token=xxx
/// Verifies the token and creates a session.
pub async fn handle_verify_magic_link(
    State(state): State<MagicLinkState>,
    Query(params): Query<VerifyMagicLinkQuery>,
) -> Result<Json<VerifyMagicLinkResponse>, AppError> {
    // 1. Hash the provided token to look up in DB
    let token_hash = nucleus_core::crypto::generate_token_hash(&params.token);

    // 2. Find stored token
    let stored = state
        .token_repo
        .find_by_hash(&token_hash, "magic_link")
        .await?
        .ok_or(AppError::Auth(AuthError::TokenInvalid))?;

    // 3. Verify: hash matches, not expired, not used
    MagicLinkService::verify_token(
        &params.token,
        &stored.token_hash,
        &stored.expires_at,
        &stored.used_at,
    )?;

    // 4. Mark token as used
    state.token_repo.mark_used(stored.id).await?;

    // 5. Find user
    let project_id = ProjectId::from_uuid(stored.project_id);
    let user_id = nucleus_core::types::UserId::from_uuid(stored.user_id);
    let user = state
        .user_repo
        .find_by_id(&project_id, &user_id)
        .await?
        .ok_or(AppError::User(UserError::NotFound))?;

    // 6. Create session
    let (_session_token, session) = state
        .session_service
        .create_session(&user.id, &project_id, DeviceInfo::default(), 3600)
        .await?;

    // 7. Issue JWT
    let jwt = state.auth_service.issue_jwt_for_user(&user, &project_id)?;

    Ok(Json(VerifyMagicLinkResponse {
        user: serde_json::to_value(&user)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("serialize error: {}", e)))?,
        jwt,
        session_token: session.id.to_string(),
    }))
}
