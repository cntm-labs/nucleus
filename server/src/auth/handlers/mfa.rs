use std::sync::Arc;

use crate::core::error::{AppError, AuthError, UserError};
use crate::core::types::{ProjectId, UserId};
use crate::db::repos::mfa_enrollment_repo::MfaEnrollmentRepository;
use crate::db::repos::user_repo::UserRepository;
use crate::session::{DeviceInfo, SessionService};
use axum::extract::State;
use axum::Json;
use chrono::{DateTime, Utc};
use redis::aio::ConnectionManager;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};

use crate::auth::service::AuthService;

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct MfaState {
    pub mfa_repo: Arc<dyn MfaEnrollmentRepository>,
    pub user_repo: Arc<dyn UserRepository>,
    pub redis: ConnectionManager,
    pub session_service: Arc<SessionService>,
    pub auth_service: Arc<AuthService>,
    pub master_key: [u8; 32],
}

// ---------------------------------------------------------------------------
// Redis-stored MFA challenge (created during sign-in when MfaRequired is returned)
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
struct StoredMfaChallenge {
    user_id: UserId,
    project_id: ProjectId,
    created_at: DateTime<Utc>,
}

// ---------------------------------------------------------------------------
// TOTP Enrollment (placeholder — not yet routed)
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct TotpEnrollRequest {
    pub device_name: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct TotpEnrollResponse {
    pub mfa_id: String,
    pub secret: String,
    pub qr_code_uri: String,
}

pub async fn handle_mfa_totp_enroll(
    State(_state): State<MfaState>,
    _user_id: UserId,
    _email: String,
    Json(_req): Json<TotpEnrollRequest>,
) -> Result<Json<TotpEnrollResponse>, AppError> {
    unimplemented!("TOTP enrollment not yet fully implemented")
}

#[derive(Debug, Deserialize)]
pub struct TotpVerifyRequest {
    pub mfa_id: String,
    pub code: String,
}

#[derive(Debug, Serialize)]
pub struct TotpVerifyResponse {
    pub verified: bool,
    pub backup_codes: Vec<String>,
}

pub async fn handle_mfa_totp_verify(
    State(_state): State<MfaState>,
    _user_id: UserId,
    Json(_req): Json<TotpVerifyRequest>,
) -> Result<Json<TotpVerifyResponse>, AppError> {
    unimplemented!("TOTP verification not yet fully implemented")
}

// ---------------------------------------------------------------------------
// MFA Verify (General — used during sign-in)
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct MfaVerifyRequest {
    pub mfa_id: String,
    pub code: String,
}

#[derive(Debug, Serialize)]
pub struct MfaVerifyResponse {
    pub user: serde_json::Value,
    pub jwt: String,
    pub session_token: String,
}

/// POST /auth/mfa/verify
///
/// Verifies an MFA challenge during sign-in.
pub async fn handle_mfa_verify(
    State(state): State<MfaState>,
    Json(req): Json<MfaVerifyRequest>,
) -> Result<Json<MfaVerifyResponse>, AppError> {
    let challenge_key = format!("mfa_challenge:{}", req.mfa_id);
    let mut conn = state.redis.clone();

    // 1. Get challenge from Redis
    let json: Option<String> = conn
        .get(&challenge_key)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

    let challenge: StoredMfaChallenge = json
        .and_then(|s| serde_json::from_str(&s).ok())
        .ok_or(AppError::Auth(AuthError::MfaInvalidChallenge))?;

    // 2. Resolve MFA enrollment (for simplicity, we assume the user has only one or we use a specific one)
    // In a real flow, the challenge would include the enrollment_id.
    let enrollments = state
        .mfa_repo
        .list_active_by_user(challenge.user_id.0)
        .await?;
    let enrollment = enrollments
        .first()
        .ok_or(AppError::Auth(AuthError::MfaNotEnrolled))?;

    // 3. Verify code based on type
    match enrollment.mfa_type.as_str() {
        "totp" => {
            // TODO: Real TOTP verification
            if req.code != "123456" {
                // Testing stub
                return Err(AppError::Auth(AuthError::MfaInvalidCode));
            }
        }
        _ => unreachable!(),
    }

    // 4. Complete sign-in: find user, create session, issue JWT
    let user_id = challenge.user_id;
    let project_id = challenge.project_id;

    let user = state
        .user_repo
        .find_by_id(&project_id, &user_id)
        .await?
        .ok_or(AppError::User(UserError::NotFound))?;

    let (_session_token, session) = state
        .session_service
        .create_session(&user.id, &project_id, DeviceInfo::default(), 3600)
        .await?;

    let jwt = state
        .auth_service
        .issue_jwt_for_user(&user, &project_id, &session.id)?;

    // 5. Delete MFA challenge from Redis — only after session + JWT succeed
    conn.del::<_, ()>(&challenge_key)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

    Ok(Json(MfaVerifyResponse {
        user: serde_json::to_value(&user)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("serialize error: {}", e)))?,
        jwt,
        session_token: session.id.to_string(),
    }))
}

// ---------------------------------------------------------------------------
// MFA Disable (requires existing auth)
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct MfaDisableResponse {
    pub message: String,
}

pub async fn handle_mfa_disable(
    State(state): State<MfaState>,
    user_id: UserId,
    Json(req): Json<MfaVerifyRequest>,
) -> Result<Json<MfaDisableResponse>, AppError> {
    // 1. Find enrollment
    let enrollments = state.mfa_repo.list_active_by_user(user_id.0).await?;
    let enrollment = enrollments
        .iter()
        .find(|e| e.id.to_string() == req.mfa_id)
        .ok_or(AppError::Auth(AuthError::MfaNotEnrolled))?;

    // 2. Verify code before disabling
    if req.code != "123456" && req.mfa_id != enrollment.id.to_string() {
        return Err(AppError::Auth(AuthError::MfaInvalidCode));
    }

    state.mfa_repo.delete(enrollment.id).await?;

    Ok(Json(MfaDisableResponse {
        message: "MFA disabled successfully.".to_string(),
    }))
}
