use async_trait::async_trait;
use chrono::{DateTime, Utc};
use nucleus_core::error::AppError;
use nucleus_core::types::{CredentialId, UserId};
use serde::Serialize;
use sqlx::postgres::PgRow;
use sqlx::{PgPool, Row};

#[derive(Debug, Clone, Serialize)]
pub struct Credential {
    pub id: CredentialId,
    pub user_id: UserId,
    pub credential_type: String,
    pub identifier: Option<String>,
    pub secret_hash: Option<String>,
    pub provider: Option<String>,
    pub provider_data: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct NewCredential {
    pub user_id: UserId,
    pub credential_type: String,
    pub identifier: Option<String>,
    pub secret_hash: Option<String>,
    pub provider: Option<String>,
    pub provider_data: Option<serde_json::Value>,
}

fn credential_from_row(row: &PgRow) -> Result<Credential, sqlx::Error> {
    Ok(Credential {
        id: CredentialId::from_uuid(row.try_get("id")?),
        user_id: UserId::from_uuid(row.try_get("user_id")?),
        credential_type: row.try_get::<String, _>("credential_type")?,
        identifier: row.try_get("identifier")?,
        secret_hash: row.try_get("secret_hash")?,
        provider: row.try_get("provider")?,
        provider_data: row.try_get("provider_data")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

#[async_trait]
pub trait CredentialRepository: Send + Sync {
    async fn create(&self, credential: &NewCredential) -> Result<Credential, AppError>;
    async fn find_by_user_and_type(
        &self,
        user_id: &UserId,
        credential_type: &str,
    ) -> Result<Vec<Credential>, AppError>;
    async fn find_by_provider_identifier(
        &self,
        credential_type: &str,
        provider: &str,
        identifier: &str,
    ) -> Result<Option<Credential>, AppError>;
    async fn update_secret(&self, id: &CredentialId, new_secret_hash: &str)
        -> Result<(), AppError>;
    async fn delete(&self, id: &CredentialId) -> Result<(), AppError>;
}

pub struct PgCredentialRepository {
    pool: PgPool,
}

impl PgCredentialRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CredentialRepository for PgCredentialRepository {
    async fn create(&self, credential: &NewCredential) -> Result<Credential, AppError> {
        let row = sqlx::query(
            r#"
            INSERT INTO credentials (user_id, credential_type, identifier, secret_hash, provider, provider_data)
            VALUES ($1, $2::credential_type, $3, $4, $5, $6)
            RETURNING *
            "#,
        )
        .bind(credential.user_id.0)
        .bind(&credential.credential_type)
        .bind(&credential.identifier)
        .bind(&credential.secret_hash)
        .bind(&credential.provider)
        .bind(&credential.provider_data)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

        credential_from_row(&row).map_err(|e| AppError::Internal(e.into()))
    }

    async fn find_by_user_and_type(
        &self,
        user_id: &UserId,
        credential_type: &str,
    ) -> Result<Vec<Credential>, AppError> {
        let rows = sqlx::query(
            "SELECT * FROM credentials WHERE user_id = $1 AND credential_type = $2::credential_type",
        )
        .bind(user_id.0)
        .bind(credential_type)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

        rows.iter()
            .map(|r| credential_from_row(r))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| AppError::Internal(e.into()))
    }

    async fn find_by_provider_identifier(
        &self,
        credential_type: &str,
        provider: &str,
        identifier: &str,
    ) -> Result<Option<Credential>, AppError> {
        let row = sqlx::query(
            r#"
            SELECT * FROM credentials
            WHERE credential_type = $1::credential_type AND provider = $2 AND identifier = $3
            "#,
        )
        .bind(credential_type)
        .bind(provider)
        .bind(identifier)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

        match row {
            Some(ref r) => Ok(Some(
                credential_from_row(r).map_err(|e| AppError::Internal(e.into()))?,
            )),
            None => Ok(None),
        }
    }

    async fn update_secret(
        &self,
        id: &CredentialId,
        new_secret_hash: &str,
    ) -> Result<(), AppError> {
        sqlx::query("UPDATE credentials SET secret_hash = $1, updated_at = now() WHERE id = $2")
            .bind(new_secret_hash)
            .bind(id.0)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Internal(e.into()))?;

        Ok(())
    }

    async fn delete(&self, id: &CredentialId) -> Result<(), AppError> {
        sqlx::query("DELETE FROM credentials WHERE id = $1")
            .bind(id.0)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Internal(e.into()))?;

        Ok(())
    }
}
