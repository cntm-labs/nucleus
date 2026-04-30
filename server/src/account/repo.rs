//! AccountRepository — full impl in Task 2.

use crate::core::types::AccountId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: AccountId,
}

pub struct NewAccount;
pub struct UpdateAccount;

#[async_trait::async_trait]
pub trait AccountRepository: Send + Sync {}

pub struct PgAccountRepository;
