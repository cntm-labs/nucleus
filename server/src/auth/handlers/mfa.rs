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
use uuid::Uuid;

use crate::auth::mfa::MfaService;
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
    user_id: Uuid,
    project_id: Uuid,
    created_at: DateTime<Utc>,
}

// ---------------------------------------------------------------------------
// TOTP Enrollment (placeholder — not yet routed)
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct TotpEnrollRequest {
    pub issuer: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct TotpEnrollResponse {
    pub totp_uri: String,
    pub secret_base32: String,
    pub backup_codes: Vec<String>,
}

/// POST /users/me/mfa/totp/enroll
pub async fn handle_mfa_totp_enroll(
    State(state): State<MfaState>,
    user_id: UserId,
    user_email: String,
    Json(req): Json<TotpEnrollRequest>,
) -> Result<Json<TotpEnrollResponse>, AppError> {
    let issuer = req.issuer.as_deref().unwrap_or("Nucleus");

    let enrollment = MfaService::enroll_totp(&user_email, issuer, &state.master_key)?;
    let backup_codes = MfaService::generate_backup_codes();
    let backup_codes_enc = MfaService::encrypt_backup_codes(&backup_codes, &state.master_key)?;

    state
        .mfa_repo
        .create(
            user_id.0,
            "totp",
            Some(&enrollment.secret_enc),
            Some(&backup_codes_enc),
        )
        .await?;

    Ok(Json(TotpEnrollResponse {
        totp_uri: enrollment.totp_uri,
        secret_base32: enrollment.secret_base32,
        backup_codes,
    }))
}

// ---------------------------------------------------------------------------
// TOTP Verify (complete enrollment — not yet routed)
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct TotpVerifyRequest {
    pub code: String,
}

#[derive(Debug, Serialize)]
pub struct TotpVerifyResponse {
    pub verified: bool,
}

/// POST /users/me/mfa/totp/verify
pub async fn handle_mfa_totp_verify(
    State(state): State<MfaState>,
    user_id: UserId,
    Json(req): Json<TotpVerifyRequest>,
) -> Result<Json<TotpVerifyResponse>, AppError> {
    // Find pending (unverified) enrollment
    // For TOTP verify during enrollment, we look for any enrollment for this user
    let enrollment = state
        .mfa_repo
        .find_active_by_user(user_id.0, "totp")
        .await?;

    // If no verified enrollment, check for unverified one
    // For simplicity, we check if there's any enrollment and verify against it
    let enrollment = match enrollment {
        Some(e) => e,
        None => {
            return Err(AppError::Auth(AuthError::MfaInvalidCode));
        }
    };

    let secret_enc = enrollment
        .secret_enc
        .as_deref()
        .ok_or(AppError::Internal(anyhow::anyhow!("Missing TOTP secret")))?;

    let valid = MfaService::verify_totp(&req.code, secret_enc, &state.master_key)?;

    if valid {
        state.mfa_repo.update_last_used(enrollment.id).await?;
    }

    Ok(Json(TotpVerifyResponse { verified: valid }))
}

// ---------------------------------------------------------------------------
// MFA Verify (during sign-in flow)
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct MfaVerifyRequest {
    pub mfa_id: String,
    /// "totp" or "backup_code"
    pub method: String,
    pub code: String,
}

#[derive(Debug, Serialize)]
pub struct MfaVerifyResponse {
    pub user: serde_json::Value,
    pub jwt: String,
    pub session_token: String,
}

/// POST /auth/mfa/verify
/// Verify MFA during the sign-in flow (after password check returned MfaRequired).
pub async fn handle_mfa_verify(
    State(state): State<MfaState>,
    Json(req): Json<MfaVerifyRequest>,
) -> Result<Json<MfaVerifyResponse>, AppError> {
    // Validate method
    match req.method.as_str() {
        "totp" | "backup_code" => {}
        _ => return Err(AppError::Auth(AuthError::MfaInvalidCode)),
    }

    // 1. Load MFA challenge from Redis
    let challenge_key = format!("mfa_challenge:{}", req.mfa_id);
    let mut conn = state.redis.clone();

    let challenge_json: Option<String> = conn
        .get(&challenge_key)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

    let challenge_json = challenge_json.ok_or(AppError::Auth(AuthError::TokenExpired))?;
    let challenge: StoredMfaChallenge = serde_json::from_str(&challenge_json)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("deserialize error: {}", e)))?;

    // 2. Load user's active MFA enrollment
    let enrollment = state
        .mfa_repo
        .find_active_by_user(challenge.user_id, "totp")
        .await?
        .ok_or(AppError::Auth(AuthError::MfaInvalidCode))?;

    // 3. Verify based on method
    match req.method.as_str() {
        "totp" => {
            let secret_enc = enrollment
                .secret_enc
                .as_deref()
                .ok_or(AppError::Internal(anyhow::anyhow!("Missing TOTP secret")))?;

            let valid = MfaService::verify_totp(&req.code, secret_enc, &state.master_key)?;
            if !valid {
                return Err(AppError::Auth(AuthError::MfaInvalidCode));
            }
            state.mfa_repo.update_last_used(enrollment.id).await?;
        }
        "backup_code" => {
            let codes_enc = enrollment
                .backup_codes_enc
                .as_deref()
                .ok_or(AppError::Auth(AuthError::MfaInvalidCode))?;

            let (valid, new_enc) =
                MfaService::verify_backup_code(&req.code, codes_enc, &state.master_key)?;

            if !valid {
                return Err(AppError::Auth(AuthError::MfaInvalidCode));
            }

            state
                .mfa_repo
                .update_backup_codes(enrollment.id, &new_enc)
                .await?;
        }
        _ => unreachable!(),
    }

    // 4. Complete sign-in: find user, create session, issue JWT
    let user_id = UserId::from_uuid(challenge.user_id);
    let project_id = ProjectId::from_uuid(challenge.project_id);

    let user = state
        .user_repo
        .find_by_id(&project_id, &user_id)
        .await?
        .ok_or(AppError::User(UserError::NotFound))?;

    let (_session_token, session) = state
        .session_service
        .create_session(&user.id, &project_id, DeviceInfo::default(), 3600)
        .await?;

    let jwt = state.auth_service.issue_jwt_for_user(&user, &project_id)?;

    // 5. Delete MFA challenge from Redis only after session is successfully created,
    //    so the user can retry if session creation fails.
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
// Disable MFA (not yet routed)
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct MfaDisableResponse {
    pub message: String,
}

/// DELETE /users/me/mfa/:enrollment_id
pub async fn handle_mfa_disable(
    State(state): State<MfaState>,
    user_id: UserId,
    enrollment_id: Uuid,
) -> Result<Json<MfaDisableResponse>, AppError> {
    let enrollment = state
        .mfa_repo
        .find_active_by_user(user_id.0, "totp")
        .await?
        .ok_or(AppError::Auth(AuthError::MfaInvalidCode))?;

    if enrollment.id != enrollment_id {
        return Err(AppError::Auth(AuthError::MfaInvalidCode));
    }

    state.mfa_repo.delete(enrollment.id).await?;

    Ok(Json(MfaDisableResponse {
        message: "MFA disabled successfully.".to_string(),
    }))
}
