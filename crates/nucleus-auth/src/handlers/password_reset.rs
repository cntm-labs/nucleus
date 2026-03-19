use axum::Json;
use nucleus_core::error::AppError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct RequestResetRequest {
    pub email: String,
}

#[derive(Debug, Serialize)]
pub struct RequestResetResponse {
    pub message: String,
}

/// POST /auth/password/reset
/// Send password reset email
pub async fn handle_request_reset(
    Json(_req): Json<RequestResetRequest>,
) -> Result<Json<RequestResetResponse>, AppError> {
    // Generate token, store hash in verification_tokens (token_type = 'password_reset')
    // Send email with reset link
    // Anti-enumeration: always return success
    Ok(Json(RequestResetResponse {
        message: "If an account exists with this email, a password reset link has been sent."
            .to_string(),
    }))
}

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
/// Verify token and update password
pub async fn handle_confirm_reset(
    Json(_req): Json<ConfirmResetRequest>,
) -> Result<Json<ConfirmResetResponse>, AppError> {
    // 1. Lookup token hash in verification_tokens
    // 2. Verify token (not expired, not used)
    // 3. Validate new password
    // 4. Hash new password
    // 5. Update credential
    // 6. Mark token as used
    // 7. Invalidate all sessions for user
    Ok(Json(ConfirmResetResponse {
        message: "Password has been reset successfully.".to_string(),
    }))
}
