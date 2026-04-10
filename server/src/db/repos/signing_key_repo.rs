use crate::core::error::AppError;
use async_trait::async_trait;
use sqlx::postgres::PgRow;
use sqlx::{PgPool, Row};
use uuid::Uuid;

pub struct SigningKeyRow {
    pub id: Uuid,
    pub project_id: Uuid,
    pub algorithm: String,
    pub public_key: String,
    pub private_key_enc: String,
    pub is_current: bool,
}

fn signing_key_from_row(row: &PgRow) -> Result<SigningKeyRow, sqlx::Error> {
    Ok(SigningKeyRow {
        id: row.try_get("id")?,
        project_id: row.try_get("project_id")?,
        algorithm: row.try_get("algorithm")?,
        public_key: row.try_get("public_key")?,
        private_key_enc: row.try_get("private_key_enc")?,
        is_current: row.try_get("is_current")?,
    })
}

#[async_trait]
pub trait SigningKeyRepository: Send + Sync {
    async fn find_current(&self, project_id: &Uuid) -> Result<Option<SigningKeyRow>, AppError>;
    async fn create(
        &self,
        project_id: &Uuid,
        algorithm: &str,
        public_key: &str,
        private_key_enc: &str,
    ) -> Result<SigningKeyRow, AppError>;
}

pub struct PgSigningKeyRepository {
    pool: PgPool,
}

impl PgSigningKeyRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SigningKeyRepository for PgSigningKeyRepository {
    async fn find_current(&self, project_id: &Uuid) -> Result<Option<SigningKeyRow>, AppError> {
        let row = sqlx::query(
            "SELECT id, project_id, algorithm, public_key, private_key_enc, is_current
             FROM signing_keys WHERE project_id = $1 AND is_current = true",
        )
        .bind(project_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

        match row {
            Some(ref r) => Ok(Some(
                signing_key_from_row(r).map_err(|e| AppError::Internal(e.into()))?,
            )),
            None => Ok(None),
        }
    }

    async fn create(
        &self,
        project_id: &Uuid,
        algorithm: &str,
        public_key: &str,
        private_key_enc: &str,
    ) -> Result<SigningKeyRow, AppError> {
        let row = sqlx::query(
            r#"INSERT INTO signing_keys (project_id, algorithm, public_key, private_key_enc, is_current)
               VALUES ($1, $2, $3, $4, true)
               RETURNING id, project_id, algorithm, public_key, private_key_enc, is_current"#,
        )
        .bind(project_id)
        .bind(algorithm)
        .bind(public_key)
        .bind(private_key_enc)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

        signing_key_from_row(&row).map_err(|e| AppError::Internal(e.into()))
    }
}
