use axum::extract::Query;
use axum::Json;
use nucleus_core::error::AppError;
use serde::{Deserialize, Serialize};

// Service used by this module (called once DB integration is wired):
// use crate::magic_link::MagicLinkService;

#[derive(Debug, Deserialize)]
pub struct SendMagicLinkRequest {
    pub email: String,
    pub redirect_url: String,
}

#[derive(Debug, Serialize)]
pub struct SendMagicLinkResponse {
    pub message: String,
}

/// POST /auth/sign-in/magic-link
/// Generates a magic link and "sends" it (in production, via email service).
/// Rate limit: 1 per minute per email.
/// Always returns 200 (anti-enumeration: don't reveal if email exists).
pub async fn handle_send_magic_link(
    Json(_body): Json<SendMagicLinkRequest>,
) -> Result<Json<SendMagicLinkResponse>, AppError> {
    // TODO: Look up user by email, generate token, store hash in
    // verification_tokens table, send email with magic link URL.
    // For anti-enumeration, always return success even if email not found.
    Ok(Json(SendMagicLinkResponse {
        message: "If an account exists with this email, a magic link has been sent.".to_string(),
    }))
}

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
    Query(_params): Query<VerifyMagicLinkQuery>,
) -> Result<Json<VerifyMagicLinkResponse>, AppError> {
    // Flow:
    // 1. Hash the provided token to look up in verification_tokens table
    // 2. Retrieve the stored record (token_hash, expires_at, used_at, user_id)
    // 3. Call MagicLinkService::verify_token(token, stored_hash, expires_at, used_at)
    // 4. Mark token as used in DB
    // 5. Create session + issue JWT for the user
    //
    // Requires: VerificationTokenRepository (not yet implemented)
    Err(AppError::Internal(anyhow::anyhow!(
        "Magic link verification requires database integration (VerificationTokenRepository)"
    )))
}
