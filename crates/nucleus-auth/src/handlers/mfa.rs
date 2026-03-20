use axum::Json;
use nucleus_core::error::{AppError, AuthError};
use serde::{Deserialize, Serialize};

use crate::mfa::MfaService;

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
    Json(req): Json<TotpEnrollRequest>,
) -> Result<Json<TotpEnrollResponse>, AppError> {
    // In production, the authenticated user's email and the project's encryption
    // key would come from request extensions / app state. For now, we use
    // placeholder values to demonstrate the full enrollment flow.
    let issuer = req.issuer.as_deref().unwrap_or("Nucleus");

    // TODO: extract from authenticated user context once middleware is wired
    let user_email = "placeholder@pending-auth-middleware.local";

    // TODO: extract from project config / AppState once wired
    let encryption_key = nucleus_core::crypto::generate_encryption_key();

    // 1. Generate TOTP enrollment (secret + QR URI)
    let enrollment = MfaService::enroll_totp(user_email, issuer, &encryption_key)?;

    // 2. Generate backup codes
    let backup_codes = MfaService::generate_backup_codes();

    // 3. In production: store enrollment.secret_enc + encrypted backup codes
    //    in mfa_enrollments table with status = 'pending_verification'
    //    let _encrypted_backups = MfaService::encrypt_backup_codes(&backup_codes, &encryption_key)?;

    Ok(Json(TotpEnrollResponse {
        totp_uri: enrollment.totp_uri,
        secret_base32: enrollment.secret_base32,
        backup_codes,
    }))
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
    // Flow:
    // 1. Get authenticated user from request extensions
    // 2. Load pending TOTP enrollment from mfa_enrollments table
    // 3. Call MfaService::verify_totp(code, secret_enc, encryption_key)
    // 4. If valid, update enrollment status to 'active' in DB
    // 5. Return result
    //
    // Requires: MfaEnrollmentRepository + auth middleware (not yet wired)
    Err(AppError::Internal(anyhow::anyhow!(
        "TOTP verification requires database integration (MfaEnrollmentRepository)"
    )))
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
    Json(req): Json<MfaVerifyRequest>,
) -> Result<Json<MfaVerifyResponse>, AppError> {
    // Validate method is supported
    match req.method.as_str() {
        "totp" | "backup_code" => {}
        _ => {
            return Err(AppError::Auth(AuthError::MfaInvalidCode));
        }
    }

    // Flow:
    // 1. Load MFA challenge by mfa_id from Redis (stored during sign-in when
    //    AuthError::MfaRequired was returned). Contains user_id + project_id.
    // 2. Load user's active MFA enrollment from DB
    // 3. Based on method:
    //    - "totp": call MfaService::verify_totp(code, secret_enc, encryption_key)
    //    - "backup_code": call MfaService::verify_backup_code(code, encrypted_codes, key)
    //      and update remaining codes in DB
    // 4. If valid, complete sign-in: create session, issue JWT
    // 5. If invalid, return MfaInvalidCode error
    // 6. Delete MFA challenge from Redis
    //
    // Requires: Redis + MfaEnrollmentRepository + SessionService (not yet wired)
    Err(AppError::Internal(anyhow::anyhow!(
        "MFA verification requires Redis and database integration"
    )))
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
    // Flow:
    // 1. Get authenticated user from request extensions
    // 2. Load enrollment by ID, verify it belongs to the user
    // 3. Delete MFA enrollment from mfa_enrollments table
    // 4. Return confirmation
    //
    // Requires: MfaEnrollmentRepository + auth middleware (not yet wired)
    Err(AppError::Internal(anyhow::anyhow!(
        "MFA disable requires database integration (MfaEnrollmentRepository)"
    )))
}
