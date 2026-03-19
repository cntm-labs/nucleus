use axum::Json;
use nucleus_core::error::AppError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct SendOtpRequest {
    pub email_or_phone: String,
}

#[derive(Debug, Serialize)]
pub struct SendOtpResponse {
    pub message: String,
}

/// POST /auth/sign-in/otp/send
pub async fn handle_send_otp(
    Json(_req): Json<SendOtpRequest>,
) -> Result<Json<SendOtpResponse>, AppError> {
    // Generate OTP, store hash in Redis (key: otp:{project}:{email}:{purpose})
    // "Send" via email/SMS service (placeholder for now)
    // Anti-enumeration: always return success
    Ok(Json(SendOtpResponse {
        message: "If an account exists, a verification code has been sent.".to_string(),
    }))
}

#[derive(Debug, Deserialize)]
pub struct VerifyOtpRequest {
    pub email_or_phone: String,
    pub code: String,
}

#[derive(Debug, Serialize)]
pub struct VerifyOtpResponse {
    pub user: serde_json::Value,
    pub jwt: String,
    pub session_token: String,
}

/// POST /auth/sign-in/otp/verify
pub async fn handle_verify_otp(
    Json(_req): Json<VerifyOtpRequest>,
) -> Result<Json<VerifyOtpResponse>, AppError> {
    // Lookup OTP in Redis, verify code, increment attempts
    // If valid: find/create user, create session, issue JWT
    todo!()
}
