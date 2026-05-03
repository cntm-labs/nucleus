use std::sync::Arc;

use axum::{extract::State, Json};

use crate::account::repo::Account;
use crate::account::service::AccountService;
use crate::core::error::AppError;
use crate::middleware::account_auth::AuthAccount;

pub async fn handle_me(
    State(svc): State<Arc<AccountService>>,
    auth: AuthAccount,
) -> Result<Json<Account>, AppError> {
    let account = svc.get_me(&auth.account_id).await?;
    Ok(Json(account))
}
