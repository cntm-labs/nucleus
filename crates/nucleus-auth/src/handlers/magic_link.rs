use axum::extract::Query;
use axum::Json;
use nucleus_core::error::AppError;
use serde::{Deserialize, Serialize};

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
    // TODO: Look up token hash in DB, verify with MagicLinkService::verify_token,
    // mark as used, create session + JWT, return user info.
    todo!()
}
