use axum::Json;
use nucleus_core::error::AppError;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// TOTP Enrollment
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
/// Generates a new TOTP secret + backup codes and returns the QR URI.
pub async fn handle_mfa_totp_enroll(
    Json(_req): Json<TotpEnrollRequest>,
) -> Result<Json<TotpEnrollResponse>, AppError> {
    // 1. Get authenticated user from request extensions (placeholder)
    // 2. Call MfaService::enroll_totp(user.email, issuer, encryption_key)
    // 3. Generate backup codes via MfaService::generate_backup_codes()
    // 4. Encrypt & store secret_enc + backup codes in DB (pending verification)
    // 5. Return QR URI + secret + backup codes to user
    todo!()
}

// ---------------------------------------------------------------------------
// TOTP Verify (complete enrollment)
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
/// Verify a TOTP code to activate the enrollment.
pub async fn handle_mfa_totp_verify(
    Json(_req): Json<TotpVerifyRequest>,
) -> Result<Json<TotpVerifyResponse>, AppError> {
    // 1. Get authenticated user
    // 2. Load pending TOTP enrollment from DB
    // 3. Call MfaService::verify_totp(code, secret_enc, encryption_key)
    // 4. If valid, mark enrollment as active
    // 5. Return result
    todo!()
}

// ---------------------------------------------------------------------------
// MFA Verify (during sign-in flow)
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct MfaVerifyRequest {
    pub mfa_id: String,
    pub code: String,
    /// "totp" or "backup_code"
    pub method: String,
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
    Json(_req): Json<MfaVerifyRequest>,
) -> Result<Json<MfaVerifyResponse>, AppError> {
    // 1. Load MFA challenge by mfa_id from Redis/DB
    // 2. Based on method:
    //    - "totp": call MfaService::verify_totp(code, secret_enc, key)
    //    - "backup_code": call MfaService::verify_backup_code(code, encrypted_codes, key)
    // 3. If valid, complete sign-in: create session, issue JWT
    // 4. If invalid, return MfaInvalidCode error
    todo!()
}

// ---------------------------------------------------------------------------
// Disable MFA
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct MfaDisableResponse {
    pub message: String,
}

/// DELETE /users/me/mfa/:enrollment_id
/// Disable an MFA method for the authenticated user.
pub async fn handle_mfa_disable() -> Result<Json<MfaDisableResponse>, AppError> {
    // 1. Get authenticated user
    // 2. Verify the enrollment belongs to the user
    // 3. Delete MFA enrollment from DB
    // 4. Return confirmation
    todo!()
}
