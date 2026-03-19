use async_trait::async_trait;
use chrono::{DateTime, Utc};
use nucleus_core::error::AppError;
use nucleus_core::types::{ApiKeyId, ProjectId};
use serde::Serialize;
use sqlx::postgres::PgRow;
use sqlx::{PgPool, Row};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
pub struct ApiKey {
    pub id: ApiKeyId,
    pub project_id: ProjectId,
    pub key_type: String,
    pub key_hash: String,
    pub key_prefix: String,
    pub environment: String,
    pub label: Option<String>,
    pub scopes: Vec<String>,
    pub rate_limit: Option<i32>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
}

pub struct NewApiKey {
    pub project_id: ProjectId,
    pub key_type: String,
    pub key_hash: String,
    pub key_prefix: String,
    pub environment: Option<String>,
    pub label: Option<String>,
    pub scopes: Vec<String>,
    pub rate_limit: Option<i32>,
    pub expires_at: Option<DateTime<Utc>>,
}

fn api_key_from_row(row: &PgRow) -> Result<ApiKey, sqlx::Error> {
    Ok(ApiKey {
        id: ApiKeyId::from_uuid(row.try_get::<Uuid, _>("id")?),
        project_id: ProjectId::from_uuid(row.try_get::<Uuid, _>("project_id")?),
        key_type: row.try_get::<String, _>("key_type")?,
        key_hash: row.try_get("key_hash")?,
        key_prefix: row.try_get("key_prefix")?,
        environment: row.try_get::<String, _>("environment")?,
        label: row.try_get("label")?,
        scopes: row.try_get("scopes")?,
        rate_limit: row.try_get("rate_limit")?,
        last_used_at: row.try_get("last_used_at")?,
        expires_at: row.try_get("expires_at")?,
        created_at: row.try_get("created_at")?,
        revoked_at: row.try_get("revoked_at")?,
    })
}

#[async_trait]
pub trait ApiKeyRepository: Send + Sync {
    async fn create(&self, api_key: &NewApiKey) -> Result<ApiKey, AppError>;
    async fn find_by_prefix(&self, prefix: &str) -> Result<Option<ApiKey>, AppError>;
    async fn find_by_project(&self, project_id: &ProjectId) -> Result<Vec<ApiKey>, AppError>;
    async fn revoke(&self, id: &ApiKeyId) -> Result<(), AppError>;
    async fn update_last_used(&self, id: &ApiKeyId) -> Result<(), AppError>;
}

pub struct PgApiKeyRepository {
    pool: PgPool,
}

impl PgApiKeyRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ApiKeyRepository for PgApiKeyRepository {
    async fn create(&self, api_key: &NewApiKey) -> Result<ApiKey, AppError> {
        let environment = api_key
            .environment
            .as_deref()
            .unwrap_or("development");

        let row = sqlx::query(
            r#"
            INSERT INTO api_keys (project_id, key_type, key_hash, key_prefix, environment, label, scopes, rate_limit, expires_at)
            VALUES ($1, $2::key_type, $3, $4, $5::environment_type, $6, $7, $8, $9)
            RETURNING *
            "#,
        )
        .bind(api_key.project_id.0)
        .bind(&api_key.key_type)
        .bind(&api_key.key_hash)
        .bind(&api_key.key_prefix)
        .bind(environment)
        .bind(&api_key.label)
        .bind(&api_key.scopes)
        .bind(api_key.rate_limit)
        .bind(api_key.expires_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

        api_key_from_row(&row).map_err(|e| AppError::Internal(e.into()))
    }

    async fn find_by_prefix(&self, prefix: &str) -> Result<Option<ApiKey>, AppError> {
        let row = sqlx::query(
            "SELECT * FROM api_keys WHERE key_prefix = $1 AND revoked_at IS NULL",
        )
        .bind(prefix)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

        match row {
            Some(ref r) => Ok(Some(
                api_key_from_row(r).map_err(|e| AppError::Internal(e.into()))?,
            )),
            None => Ok(None),
        }
    }

    async fn find_by_project(&self, project_id: &ProjectId) -> Result<Vec<ApiKey>, AppError> {
        let rows = sqlx::query("SELECT * FROM api_keys WHERE project_id = $1 ORDER BY created_at DESC")
            .bind(project_id.0)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AppError::Internal(e.into()))?;

        rows.iter()
            .map(|r| api_key_from_row(r))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| AppError::Internal(e.into()))
    }

    async fn revoke(&self, id: &ApiKeyId) -> Result<(), AppError> {
        sqlx::query("UPDATE api_keys SET revoked_at = now() WHERE id = $1")
            .bind(id.0)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Internal(e.into()))?;

        Ok(())
    }

    async fn update_last_used(&self, id: &ApiKeyId) -> Result<(), AppError> {
        sqlx::query("UPDATE api_keys SET last_used_at = now() WHERE id = $1")
            .bind(id.0)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Internal(e.into()))?;

        Ok(())
    }
}
