use std::sync::Arc;

use axum::{extract::State, Json};
use serde::Serialize;

use crate::account::service::AccountService;
use crate::core::error::AppError;
use crate::middleware::account_auth::AuthAccount;

#[derive(Debug, Serialize)]
pub struct SignOutResponse {
    pub success: bool,
}

/// Sign out — for v1.0 we trust JWT short TTL (5 min) for revocation.
/// Explicit session revoke via jti→SessionId mapping is deferred to v1.1
/// (would need a parallel index in Redis). For now the frontend simply
/// clears its localStorage token; the JWT expires within ~5 minutes anyway.
pub async fn handle_sign_out(
    State(_svc): State<Arc<AccountService>>,
    _auth: AuthAccount,
) -> Result<Json<SignOutResponse>, AppError> {
    Ok(Json(SignOutResponse { success: true }))
}
