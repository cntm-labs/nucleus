use crate::core::error::AppError;
use crate::core::types::UserId;
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::auth::passkey::PasskeyService;

// ── Registration Begin ──────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct PasskeyRegisterBeginRequest {
    pub user_id: String,
    pub email: String,
    pub display_name: String,
}

#[derive(Debug, Serialize)]
pub struct PasskeyRegisterBeginResponse {
    pub options: serde_json::Value, // RegistrationOptions serialized
}

/// POST /auth/passkey/register/begin
///
/// Initiates a passkey registration ceremony. Returns WebAuthn
/// `PublicKeyCredentialCreationOptions` for the client to pass to
/// `navigator.credentials.create()`.
pub async fn handle_passkey_register_begin(
    rp_name: &str,
    rp_id: &str,
    Json(req): Json<PasskeyRegisterBeginRequest>,
) -> Result<Json<PasskeyRegisterBeginResponse>, AppError> {
    // 1. Parse and validate user_id
    let user_id: UserId = req
        .user_id
        .parse()
        .map_err(|_| AppError::Internal(anyhow::anyhow!("Invalid user_id format")))?;

    // 2. Create PasskeyService with project RP config
    let service = PasskeyService::new(rp_name, rp_id);

    // 3. Call begin_registration to generate challenge + options
    let (options, _challenge) =
        service.begin_registration(&user_id, &req.email, &req.display_name)?;

    // 4. In production: store challenge in Redis with TTL (5 min)
    //    keyed by challenge.challenge_id for retrieval in register_finish
    //    redis.set_ex(challenge.challenge_id, serialize(challenge), 300)

    // 5. Serialize registration options for the client
    let options_json = serde_json::to_value(&options)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to serialize options: {}", e)))?;

    Ok(Json(PasskeyRegisterBeginResponse {
        options: options_json,
    }))
}

// ── Registration Finish ─────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct PasskeyRegisterFinishRequest {
    pub challenge_id: String,
    pub credential: serde_json::Value, // AuthenticatorAttestationResponse from client
}

#[derive(Debug, Serialize)]
pub struct PasskeyRegisterFinishResponse {
    pub credential_id: String,
    pub message: String,
}

/// POST /auth/passkey/register/finish
///
/// Completes the passkey registration ceremony. Validates the attestation
/// response from the authenticator, extracts the public key, and stores
/// the credential.
pub async fn handle_passkey_register_finish(
    Json(_req): Json<PasskeyRegisterFinishRequest>,
) -> Result<Json<PasskeyRegisterFinishResponse>, AppError> {
    // Flow:
    // 1. Retrieve challenge from Redis by challenge_id
    // 2. Call PasskeyService::verify_challenge_not_expired(&challenge)
    // 3. Verify attestation response against challenge (webauthn-rs integration point):
    //    - Parse clientDataJSON, verify type/origin/challenge
    //    - Parse attestationObject, extract public key
    // 4. Store PasskeyCredential in passkey_credentials table
    // 5. Delete challenge from Redis
    //
    // Requires: Redis + PasskeyCredentialRepository + WebAuthn attestation
    //           verification (not yet wired)
    Err(AppError::Internal(anyhow::anyhow!(
        "Passkey registration finish requires Redis and database integration"
    )))
}

// ── Authentication Begin ────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct PasskeyAuthBeginRequest {
    pub email: Option<String>, // Optional for discoverable credentials
}

#[derive(Debug, Serialize)]
pub struct PasskeyAuthBeginResponse {
    pub options: serde_json::Value, // AuthenticationOptions serialized
}

/// POST /auth/passkey/authenticate/begin
///
/// Initiates a passkey authentication ceremony. Returns WebAuthn
/// `PublicKeyCredentialRequestOptions` for the client to pass to
/// `navigator.credentials.get()`.
pub async fn handle_passkey_auth_begin(
    Json(_req): Json<PasskeyAuthBeginRequest>,
) -> Result<Json<PasskeyAuthBeginResponse>, AppError> {
    // Flow:
    // 1. If email provided, look up user and their passkey credentials from DB
    // 2. Create PasskeyService with project RP config
    // 3. Call service.begin_authentication(&credentials) to generate challenge
    // 4. Store challenge in Redis with TTL (5 min)
    // 5. Return authentication options to client
    //
    // For discoverable credentials (no email), return challenge with empty
    // allow_credentials list so the authenticator can offer all available keys.
    //
    // Requires: Redis + PasskeyCredentialRepository (not yet wired)
    Err(AppError::Internal(anyhow::anyhow!(
        "Passkey authentication begin requires Redis and database integration"
    )))
}

// ── Authentication Finish ───────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct PasskeyAuthFinishRequest {
    pub challenge_id: String,
    pub credential: serde_json::Value, // AuthenticatorAssertionResponse from client
}

#[derive(Debug, Serialize)]
pub struct PasskeyAuthFinishResponse {
    pub user: serde_json::Value,
    pub jwt: String,
    pub session_token: String,
}

/// POST /auth/passkey/authenticate/finish
///
/// Completes the passkey authentication ceremony. Validates the assertion
/// response, verifies the signature against the stored public key, updates
/// the sign count, and issues a session.
pub async fn handle_passkey_auth_finish(
    Json(_req): Json<PasskeyAuthFinishRequest>,
) -> Result<Json<PasskeyAuthFinishResponse>, AppError> {
    // Flow:
    // 1. Retrieve challenge from Redis by challenge_id
    // 2. Call PasskeyService::verify_challenge_not_expired(&challenge)
    // 3. Verify assertion response (webauthn-rs integration point):
    //    - Parse clientDataJSON, verify type/origin/challenge
    //    - Look up credential by credential ID in DB
    //    - Verify signature against stored public key
    //    - Check sign_count is greater than stored value (clone detection)
    // 4. Update sign_count on the credential in DB
    // 5. Create session via SessionService, issue JWT via AuthService
    // 6. Delete challenge from Redis
    //
    // Requires: Redis + PasskeyCredentialRepository + SessionService +
    //           AuthService + WebAuthn assertion verification (not yet wired)
    Err(AppError::Internal(anyhow::anyhow!(
        "Passkey authentication finish requires Redis and database integration"
    )))
}
