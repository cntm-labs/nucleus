use axum::Json;
use nucleus_core::error::AppError;
use serde::{Deserialize, Serialize};

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
    Json(_req): Json<PasskeyRegisterBeginRequest>,
) -> Result<Json<PasskeyRegisterBeginResponse>, AppError> {
    // 1. Validate user exists and is authenticated
    // 2. Create PasskeyService with project RP config
    // 3. Call begin_registration, store challenge in Redis
    // 4. Return registration options to client
    todo!()
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
    // 1. Retrieve challenge from Redis by challenge_id
    // 2. Verify challenge not expired
    // 3. Verify attestation response (webauthn-rs integration point)
    // 4. Store PasskeyCredential in database
    // 5. Delete challenge from Redis
    todo!()
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
    // 1. If email provided, look up user's passkey credentials
    // 2. Create PasskeyService with project RP config
    // 3. Call begin_authentication, store challenge in Redis
    // 4. Return authentication options to client
    todo!()
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
    // 1. Retrieve challenge from Redis by challenge_id
    // 2. Verify challenge not expired
    // 3. Verify assertion response (webauthn-rs integration point)
    // 4. Update sign_count on the credential
    // 5. Create session, issue JWT
    // 6. Delete challenge from Redis
    todo!()
}
