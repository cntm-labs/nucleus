use crate::core::error::AppError;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::postgres::PgRow;
use sqlx::{PgPool, Row};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct VerificationToken {
    pub id: Uuid,
    pub project_id: Uuid,
    pub user_id: Uuid,
    pub token_type: String,
    pub token_hash: String,
    pub redirect_url: Option<String>,
    pub expires_at: DateTime<Utc>,
    pub used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

fn token_from_row(row: &PgRow) -> Result<VerificationToken, sqlx::Error> {
    Ok(VerificationToken {
        id: row.try_get("id")?,
        project_id: row.try_get("project_id")?,
        user_id: row.try_get("user_id")?,
        token_type: row.try_get("token_type")?,
        token_hash: row.try_get("token_hash")?,
        redirect_url: row.try_get("redirect_url")?,
        expires_at: row.try_get("expires_at")?,
        used_at: row.try_get("used_at")?,
        created_at: row.try_get("created_at")?,
    })
}

#[async_trait]
pub trait VerificationTokenRepository: Send + Sync {
    async fn create(
        &self,
        user_id: Uuid,
        project_id: Uuid,
        token_type: &str,
        token_hash: &str,
        redirect_url: Option<&str>,
        expires_at: DateTime<Utc>,
    ) -> Result<VerificationToken, AppError>;

    async fn find_by_hash(
        &self,
        token_hash: &str,
        token_type: &str,
    ) -> Result<Option<VerificationToken>, AppError>;

    async fn mark_used(&self, id: Uuid) -> Result<(), AppError>;
}

pub struct PgVerificationTokenRepository {
    pool: PgPool,
}

impl PgVerificationTokenRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl VerificationTokenRepository for PgVerificationTokenRepository {
    async fn create(
        &self,
        user_id: Uuid,
        project_id: Uuid,
        token_type: &str,
        token_hash: &str,
        redirect_url: Option<&str>,
        expires_at: DateTime<Utc>,
    ) -> Result<VerificationToken, AppError> {
        let row = sqlx::query(
            r#"
            INSERT INTO verification_tokens (user_id, project_id, token_type, token_hash, redirect_url, expires_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, project_id, user_id, token_type, token_hash, redirect_url, expires_at, used_at, created_at
            "#,
        )
        .bind(user_id)
        .bind(project_id)
        .bind(token_type)
        .bind(token_hash)
        .bind(redirect_url)
        .bind(expires_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

        token_from_row(&row).map_err(|e| AppError::Internal(e.into()))
    }

    async fn find_by_hash(
        &self,
        token_hash: &str,
        token_type: &str,
    ) -> Result<Option<VerificationToken>, AppError> {
        let row = sqlx::query(
            r#"
            SELECT id, project_id, user_id, token_type, token_hash, redirect_url, expires_at, used_at, created_at
            FROM verification_tokens
            WHERE token_hash = $1 AND token_type = $2
            "#,
        )
        .bind(token_hash)
        .bind(token_type)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

        match row {
            Some(r) => Ok(Some(
                token_from_row(&r).map_err(|e| AppError::Internal(e.into()))?,
            )),
            None => Ok(None),
        }
    }

    async fn mark_used(&self, id: Uuid) -> Result<(), AppError> {
        sqlx::query("UPDATE verification_tokens SET used_at = now() WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Internal(e.into()))?;
        Ok(())
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use std::sync::Mutex;

    pub(crate) struct MockVerificationTokenRepo {
        tokens: Mutex<Vec<VerificationToken>>,
    }

    impl MockVerificationTokenRepo {
        pub(crate) fn new() -> Self {
            Self {
                tokens: Mutex::new(Vec::new()),
            }
        }
    }

    #[async_trait]
    impl VerificationTokenRepository for MockVerificationTokenRepo {
        async fn create(
            &self,
            user_id: Uuid,
            project_id: Uuid,
            token_type: &str,
            token_hash: &str,
            redirect_url: Option<&str>,
            expires_at: DateTime<Utc>,
        ) -> Result<VerificationToken, AppError> {
            let token = VerificationToken {
                id: Uuid::new_v4(),
                project_id,
                user_id,
                token_type: token_type.to_string(),
                token_hash: token_hash.to_string(),
                redirect_url: redirect_url.map(|s| s.to_string()),
                expires_at,
                used_at: None,
                created_at: Utc::now(),
            };
            self.tokens.lock().unwrap().push(token.clone());
            Ok(token)
        }

        async fn find_by_hash(
            &self,
            token_hash: &str,
            token_type: &str,
        ) -> Result<Option<VerificationToken>, AppError> {
            Ok(self
                .tokens
                .lock()
                .unwrap()
                .iter()
                .find(|t| {
                    t.token_hash == token_hash && t.token_type == token_type && t.used_at.is_none()
                })
                .cloned())
        }

        async fn mark_used(&self, id: Uuid) -> Result<(), AppError> {
            if let Some(t) = self.tokens.lock().unwrap().iter_mut().find(|t| t.id == id) {
                t.used_at = Some(Utc::now());
            }
            Ok(())
        }
    }
}
