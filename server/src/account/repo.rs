//! AccountRepository — Postgres-backed repo for the `accounts` table.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgRow;
use sqlx::{PgPool, Row};

use crate::core::error::{AccountError, AppError};
use crate::core::types::AccountId;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Account {
    pub id: AccountId,
    pub email: String,
    pub name: String,
    pub company: Option<String>,
    pub is_active: bool,
    pub email_verified: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct NewAccount {
    pub email: String,
    pub password_hash: String,
    pub name: String,
    pub company: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct UpdateAccount {
    pub name: Option<String>,
    pub company: Option<String>,
}

#[async_trait]
pub trait AccountRepository: Send + Sync {
    async fn create(&self, new: &NewAccount) -> Result<Account, AppError>;
    async fn find_by_id(&self, id: &AccountId) -> Result<Option<Account>, AppError>;
    async fn find_by_email(&self, email: &str) -> Result<Option<Account>, AppError>;
    async fn get_password_hash(&self, id: &AccountId) -> Result<String, AppError>;
    async fn mark_email_verified(&self, id: &AccountId) -> Result<(), AppError>;
    async fn update(&self, id: &AccountId, update: &UpdateAccount) -> Result<Account, AppError>;
}

fn account_from_row(row: &PgRow) -> Result<Account, sqlx::Error> {
    Ok(Account {
        id: AccountId::from_uuid(row.try_get("id")?),
        email: row.try_get("email")?,
        name: row.try_get("name")?,
        company: row.try_get("company")?,
        is_active: row.try_get("is_active")?,
        email_verified: row.try_get("email_verified")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

pub struct PgAccountRepository {
    pool: PgPool,
}

impl PgAccountRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AccountRepository for PgAccountRepository {
    async fn create(&self, new: &NewAccount) -> Result<Account, AppError> {
        let row = sqlx::query(
            r#"
            INSERT INTO accounts (email, password_hash, name, company)
            VALUES ($1, $2, $3, $4)
            RETURNING id, email, name, company, is_active, email_verified, created_at, updated_at
            "#,
        )
        .bind(&new.email)
        .bind(&new.password_hash)
        .bind(&new.name)
        .bind(&new.company)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            if let Some(db_err) = e.as_database_error() {
                if db_err.code().as_deref() == Some("23505") {
                    return AppError::Account(AccountError::EmailTaken);
                }
            }
            AppError::from(anyhow::anyhow!("create account failed: {}", e))
        })?;
        account_from_row(&row).map_err(|e| AppError::from(anyhow::anyhow!(e)))
    }

    async fn find_by_id(&self, id: &AccountId) -> Result<Option<Account>, AppError> {
        let row = sqlx::query(
            r#"
            SELECT id, email, name, company, is_active, email_verified, created_at, updated_at
            FROM accounts WHERE id = $1
            "#,
        )
        .bind(id.0)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::from(anyhow::anyhow!(e)))?;
        match row {
            Some(r) => Ok(Some(
                account_from_row(&r).map_err(|e| AppError::from(anyhow::anyhow!(e)))?,
            )),
            None => Ok(None),
        }
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<Account>, AppError> {
        let row = sqlx::query(
            r#"
            SELECT id, email, name, company, is_active, email_verified, created_at, updated_at
            FROM accounts WHERE email = $1
            "#,
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::from(anyhow::anyhow!(e)))?;
        match row {
            Some(r) => Ok(Some(
                account_from_row(&r).map_err(|e| AppError::from(anyhow::anyhow!(e)))?,
            )),
            None => Ok(None),
        }
    }

    async fn get_password_hash(&self, id: &AccountId) -> Result<String, AppError> {
        let row = sqlx::query("SELECT password_hash FROM accounts WHERE id = $1")
            .bind(id.0)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AppError::from(anyhow::anyhow!(e)))?;
        match row {
            Some(r) => Ok(r
                .try_get::<String, _>("password_hash")
                .map_err(|e| AppError::from(anyhow::anyhow!(e)))?),
            None => Err(AppError::Account(AccountError::NotFound)),
        }
    }

    async fn mark_email_verified(&self, id: &AccountId) -> Result<(), AppError> {
        sqlx::query("UPDATE accounts SET email_verified = TRUE, updated_at = now() WHERE id = $1")
            .bind(id.0)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::from(anyhow::anyhow!(e)))?;
        Ok(())
    }

    async fn update(&self, id: &AccountId, update: &UpdateAccount) -> Result<Account, AppError> {
        let row = sqlx::query(
            r#"
            UPDATE accounts
            SET name = COALESCE($2, name),
                company = COALESCE($3, company),
                updated_at = now()
            WHERE id = $1
            RETURNING id, email, name, company, is_active, email_verified, created_at, updated_at
            "#,
        )
        .bind(id.0)
        .bind(&update.name)
        .bind(&update.company)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::from(anyhow::anyhow!(e)))?
        .ok_or(AppError::Account(AccountError::NotFound))?;
        account_from_row(&row).map_err(|e| AppError::from(anyhow::anyhow!(e)))
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use std::sync::Mutex;
    use uuid::Uuid;

    pub(crate) struct MockAccountRepo {
        accounts: Mutex<Vec<(Account, String)>>,
    }

    impl MockAccountRepo {
        pub(crate) fn new() -> Self {
            Self {
                accounts: Mutex::new(Vec::new()),
            }
        }

        pub(crate) fn with_account(account: Account, password_hash: String) -> Self {
            Self {
                accounts: Mutex::new(vec![(account, password_hash)]),
            }
        }
    }

    pub(crate) fn make_account(email: &str) -> Account {
        let now = Utc::now();
        Account {
            id: AccountId::new(),
            email: email.to_string(),
            name: "Test User".to_string(),
            company: None,
            is_active: true,
            email_verified: true,
            created_at: now,
            updated_at: now,
        }
    }

    #[async_trait]
    impl AccountRepository for MockAccountRepo {
        async fn create(&self, new: &NewAccount) -> Result<Account, AppError> {
            let mut accounts = self.accounts.lock().unwrap();
            if accounts.iter().any(|(a, _)| a.email == new.email) {
                return Err(AppError::Account(AccountError::EmailTaken));
            }
            let now = Utc::now();
            let account = Account {
                id: AccountId::from_uuid(Uuid::new_v4()),
                email: new.email.clone(),
                name: new.name.clone(),
                company: new.company.clone(),
                is_active: true,
                email_verified: false,
                created_at: now,
                updated_at: now,
            };
            accounts.push((account.clone(), new.password_hash.clone()));
            Ok(account)
        }

        async fn find_by_id(&self, id: &AccountId) -> Result<Option<Account>, AppError> {
            let accounts = self.accounts.lock().unwrap();
            Ok(accounts
                .iter()
                .find(|(a, _)| a.id == *id)
                .map(|(a, _)| a.clone()))
        }

        async fn find_by_email(&self, email: &str) -> Result<Option<Account>, AppError> {
            let accounts = self.accounts.lock().unwrap();
            Ok(accounts
                .iter()
                .find(|(a, _)| a.email == email)
                .map(|(a, _)| a.clone()))
        }

        async fn get_password_hash(&self, id: &AccountId) -> Result<String, AppError> {
            let accounts = self.accounts.lock().unwrap();
            accounts
                .iter()
                .find(|(a, _)| a.id == *id)
                .map(|(_, h)| h.clone())
                .ok_or(AppError::Account(AccountError::NotFound))
        }

        async fn mark_email_verified(&self, id: &AccountId) -> Result<(), AppError> {
            let mut accounts = self.accounts.lock().unwrap();
            if let Some((a, _)) = accounts.iter_mut().find(|(a, _)| a.id == *id) {
                a.email_verified = true;
                a.updated_at = Utc::now();
            }
            Ok(())
        }

        async fn update(
            &self,
            id: &AccountId,
            update: &UpdateAccount,
        ) -> Result<Account, AppError> {
            let mut accounts = self.accounts.lock().unwrap();
            let entry = accounts
                .iter_mut()
                .find(|(a, _)| a.id == *id)
                .ok_or(AppError::Account(AccountError::NotFound))?;
            if let Some(name) = &update.name {
                entry.0.name = name.clone();
            }
            if update.company.is_some() {
                entry.0.company = update.company.clone();
            }
            entry.0.updated_at = Utc::now();
            Ok(entry.0.clone())
        }
    }

    #[tokio::test]
    async fn mock_create_then_find_by_email_round_trip() {
        // Seed an existing account via with_account + make_account, then create a new one.
        let existing = make_account("existing@example.test");
        let repo = MockAccountRepo::with_account(existing, "existing_hash".to_string());

        let new = NewAccount {
            email: "alice@example.test".to_string(),
            password_hash: "hash".to_string(),
            name: "Alice".to_string(),
            company: Some("Acme".to_string()),
        };
        let created = repo.create(&new).await.unwrap();
        assert_eq!(created.email, "alice@example.test");
        assert!(!created.email_verified);

        let found = repo
            .find_by_email("alice@example.test")
            .await
            .unwrap()
            .expect("created account should be findable");
        assert_eq!(found.id, created.id);
    }

    #[tokio::test]
    async fn mock_create_duplicate_email_returns_email_taken() {
        let repo = MockAccountRepo::new();
        let new = NewAccount {
            email: "dup@example.test".to_string(),
            password_hash: "hash".to_string(),
            name: "Dup".to_string(),
            company: None,
        };
        repo.create(&new).await.unwrap();
        let err = repo.create(&new).await.unwrap_err();
        assert!(matches!(err, AppError::Account(AccountError::EmailTaken)));
    }
}
