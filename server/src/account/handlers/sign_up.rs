use std::sync::Arc;

use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};

use crate::account::repo::Account;
use crate::account::service::AccountService;
use crate::core::error::AppError;

#[derive(Debug, Deserialize)]
pub struct SignUpRequest {
    pub email: String,
    pub password: String,
    pub name: String,
    pub company: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SignUpResponse {
    pub account: Account,
    pub message: String,
}

pub async fn handle_sign_up(
    State(svc): State<Arc<AccountService>>,
    Json(body): Json<SignUpRequest>,
) -> Result<Json<SignUpResponse>, AppError> {
    let account = svc
        .sign_up_account(
            &body.email,
            &body.password,
            &body.name,
            body.company.as_deref(),
        )
        .await?;
    Ok(Json(SignUpResponse {
        account,
        message: "Verification email sent — check your inbox to activate your account".to_string(),
    }))
}
