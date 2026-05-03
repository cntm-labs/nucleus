use std::sync::Arc;

use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};

use crate::account::repo::Account;
use crate::account::service::AccountService;
use crate::core::error::AppError;

#[derive(Debug, Deserialize)]
pub struct VerifyEmailRequest {
    pub token: String,
}

#[derive(Debug, Serialize)]
pub struct VerifyEmailResponse {
    pub account: Account,
}

pub async fn handle_verify_email(
    State(svc): State<Arc<AccountService>>,
    Json(body): Json<VerifyEmailRequest>,
) -> Result<Json<VerifyEmailResponse>, AppError> {
    let account = svc.verify_email(&body.token).await?;
    Ok(Json(VerifyEmailResponse { account }))
}
