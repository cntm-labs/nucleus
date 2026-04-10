use std::sync::Arc;

use crate::core::error::{AppError, AuthError, UserError};
use crate::core::notification::NotificationService;
use crate::core::types::ProjectId;
use crate::db::repos::user_repo::UserRepository;
use crate::session::{DeviceInfo, SessionService};
use axum::extract::State;
use axum::Json;
use chrono::{DateTime, Utc};
use redis::aio::ConnectionManager;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::auth::otp::{OtpConfig, OtpService};
use crate::auth::service::AuthService;

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct OtpState {
    pub redis: ConnectionManager,
    pub user_repo: Arc<dyn UserRepository>,
    pub session_service: Arc<SessionService>,
    pub auth_service: Arc<AuthService>,
    pub notification_service: Arc<dyn NotificationService>,
}

// ---------------------------------------------------------------------------
// Redis-stored OTP record
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
struct StoredOtp {
    code_hash: String,
    expires_at: DateTime<Utc>,
    attempts: u32,
    max_attempts: u32,
}

// ---------------------------------------------------------------------------
// Send OTP
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct SendOtpRequest {
    pub email_or_phone: String,
    pub project_id: Uuid,
}

#[derive(Debug, Serialize)]
pub struct SendOtpResponse {
    pub message: String,
}

/// POST /auth/sign-in/otp/send
pub async fn handle_send_otp(
    State(state): State<OtpState>,
    Json(body): Json<SendOtpRequest>,
) -> Result<Json<SendOtpResponse>, AppError> {
    let project_id = ProjectId::from_uuid(body.project_id);

    // Look up user — always return success (anti-enumeration)
    let user = state
        .user_repo
        .find_by_email(&project_id, &body.email_or_phone)
        .await?;

    if user.is_some() {
        let config = OtpConfig::default();
        let generated = OtpService::generate(&config);
        let key = format!("otp:{}:{}:login", body.project_id, body.email_or_phone);

        let stored = StoredOtp {
            code_hash: generated.code_hash,
            expires_at: generated.expires_at,
            attempts: 0,
            max_attempts: generated.max_attempts,
        };

        let json = serde_json::to_string(&stored)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("serialize error: {}", e)))?;

        let mut conn = state.redis.clone();
        conn.set_ex::<_, _, ()>(&key, &json, config.ttl_secs)
            .await
            .map_err(|e| AppError::Internal(e.into()))?;

        if body.email_or_phone.contains('@') {
            let _ = state
                .notification_service
                .send_email(
                    &body.email_or_phone,
                    "Your verification code",
                    &format!(
                        "<p>Your verification code is: <strong>{}</strong>. It expires in 5 minutes.</p>",
                        generated.code
                    ),
                    &format!(
                        "Your verification code is: {}. It expires in 5 minutes.",
                        generated.code
                    ),
                )
                .await;
        } else {
            let _ = state
                .notification_service
                .send_sms(
                    &body.email_or_phone,
                    &format!("Your Nucleus code: {}", generated.code),
                )
                .await;
        }
    }

    Ok(Json(SendOtpResponse {
        message: "If an account exists, a verification code has been sent.".to_string(),
    }))
}

// ---------------------------------------------------------------------------
// Verify OTP
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct VerifyOtpRequest {
    pub email_or_phone: String,
    pub code: String,
    pub project_id: Uuid,
}

#[derive(Debug, Serialize)]
pub struct VerifyOtpResponse {
    pub user: serde_json::Value,
    pub jwt: String,
    pub session_token: String,
}

/// POST /auth/sign-in/otp/verify
pub async fn handle_verify_otp(
    State(state): State<OtpState>,
    Json(body): Json<VerifyOtpRequest>,
) -> Result<Json<VerifyOtpResponse>, AppError> {
    let key = format!("otp:{}:{}:login", body.project_id, body.email_or_phone);
    let mut conn = state.redis.clone();

    // 1. Get stored OTP from Redis
    let stored_json: Option<String> = conn
        .get(&key)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

    let stored_json = stored_json.ok_or(AppError::Auth(AuthError::OtpExpired))?;

    let mut stored: StoredOtp = serde_json::from_str(&stored_json)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("deserialize error: {}", e)))?;

    // 2. Increment attempts and save back
    stored.attempts += 1;
    let updated_json = serde_json::to_string(&stored)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("serialize error: {}", e)))?;

    // Preserve remaining TTL
    let ttl: i64 = conn
        .ttl(&key)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;
    if ttl > 0 {
        conn.set_ex::<_, _, ()>(&key, &updated_json, ttl as u64)
            .await
            .map_err(|e| AppError::Internal(e.into()))?;
    }

    // 3. Verify code
    OtpService::verify(
        &body.code,
        &stored.code_hash,
        &stored.expires_at,
        stored.attempts - 1, // verify uses pre-increment count
        stored.max_attempts,
    )?;

    // 4. Delete OTP from Redis (single-use)
    conn.del::<_, ()>(&key)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

    // 5. Find user and create session
    let project_id = ProjectId::from_uuid(body.project_id);
    let user = state
        .user_repo
        .find_by_email(&project_id, &body.email_or_phone)
        .await?
        .ok_or(AppError::User(UserError::NotFound))?;

    let (_session_token, session) = state
        .session_service
        .create_session(&user.id, &project_id, DeviceInfo::default(), 3600)
        .await?;

    let jwt = state.auth_service.issue_jwt_for_user(&user, &project_id)?;

    Ok(Json(VerifyOtpResponse {
        user: serde_json::to_value(&user)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("serialize error: {}", e)))?,
        jwt,
        session_token: session.id.to_string(),
    }))
}
