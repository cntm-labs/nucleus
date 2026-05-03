use std::sync::Arc;

use axum::http::HeaderMap;
use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};

use crate::account::repo::Account;
use crate::account::service::AccountService;
use crate::core::error::AppError;
use crate::session::DeviceInfo;

#[derive(Debug, Deserialize)]
pub struct SignInRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct SignInResponse {
    pub account: Account,
    pub token: String,
}

pub async fn handle_sign_in(
    State(svc): State<Arc<AccountService>>,
    headers: HeaderMap,
    Json(body): Json<SignInRequest>,
) -> Result<Json<SignInResponse>, AppError> {
    let device = DeviceInfo {
        device_type: None,
        device_name: None,
        browser: headers
            .get("user-agent")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string()),
        ip: headers
            .get("x-forwarded-for")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.split(',').next())
            .map(|s| s.trim().to_string()),
    };

    let (account, jwt, _session) = svc
        .sign_in_account(&body.email, &body.password, device)
        .await?;
    Ok(Json(SignInResponse {
        account,
        token: jwt,
    }))
}
