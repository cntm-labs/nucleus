use async_trait::async_trait;
use chrono::{DateTime, Utc};
use nucleus_core::error::AppError;
use sqlx::postgres::PgRow;
use sqlx::{PgPool, Row};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct MfaEnrollment {
    pub id: Uuid,
    pub user_id: Uuid,
    pub mfa_type: String,
    pub secret_enc: Option<String>,
    pub phone: Option<String>,
    pub backup_codes_enc: Option<String>,
    pub verified: bool,
    pub last_used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

fn enrollment_from_row(row: &PgRow) -> Result<MfaEnrollment, sqlx::Error> {
    Ok(MfaEnrollment {
        id: row.try_get("id")?,
        user_id: row.try_get("user_id")?,
        mfa_type: row.try_get("mfa_type")?,
        secret_enc: row.try_get("secret_enc")?,
        phone: row.try_get("phone")?,
        backup_codes_enc: row.try_get("backup_codes_enc")?,
        verified: row.try_get("verified")?,
        last_used_at: row.try_get("last_used_at")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

#[async_trait]
pub trait MfaEnrollmentRepository: Send + Sync {
    async fn create(
        &self,
        user_id: Uuid,
        mfa_type: &str,
        secret_enc: Option<&str>,
        backup_codes_enc: Option<&str>,
    ) -> Result<MfaEnrollment, AppError>;

    async fn find_active_by_user(
        &self,
        user_id: Uuid,
        mfa_type: &str,
    ) -> Result<Option<MfaEnrollment>, AppError>;

    async fn mark_verified(&self, id: Uuid) -> Result<(), AppError>;

    async fn update_backup_codes(&self, id: Uuid, backup_codes_enc: &str) -> Result<(), AppError>;

    async fn update_last_used(&self, id: Uuid) -> Result<(), AppError>;

    async fn delete(&self, id: Uuid) -> Result<(), AppError>;
}

pub struct PgMfaEnrollmentRepository {
    pool: PgPool,
}

impl PgMfaEnrollmentRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl MfaEnrollmentRepository for PgMfaEnrollmentRepository {
    async fn create(
        &self,
        user_id: Uuid,
        mfa_type: &str,
        secret_enc: Option<&str>,
        backup_codes_enc: Option<&str>,
    ) -> Result<MfaEnrollment, AppError> {
        let row = sqlx::query(
            r#"
            INSERT INTO mfa_enrollments (user_id, mfa_type, secret_enc, backup_codes_enc)
            VALUES ($1, $2::mfa_type, $3, $4)
            RETURNING id, user_id, mfa_type::text, secret_enc, phone, backup_codes_enc, verified, last_used_at, created_at, updated_at
            "#,
        )
        .bind(user_id)
        .bind(mfa_type)
        .bind(secret_enc)
        .bind(backup_codes_enc)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

        enrollment_from_row(&row).map_err(|e| AppError::Internal(e.into()))
    }

    async fn find_active_by_user(
        &self,
        user_id: Uuid,
        mfa_type: &str,
    ) -> Result<Option<MfaEnrollment>, AppError> {
        let row = sqlx::query(
            r#"
            SELECT id, user_id, mfa_type::text, secret_enc, phone, backup_codes_enc, verified, last_used_at, created_at, updated_at
            FROM mfa_enrollments
            WHERE user_id = $1 AND mfa_type = $2::mfa_type AND verified = true
            "#,
        )
        .bind(user_id)
        .bind(mfa_type)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

        match row {
            Some(r) => Ok(Some(
                enrollment_from_row(&r).map_err(|e| AppError::Internal(e.into()))?,
            )),
            None => Ok(None),
        }
    }

    async fn mark_verified(&self, id: Uuid) -> Result<(), AppError> {
        sqlx::query("UPDATE mfa_enrollments SET verified = true, updated_at = now() WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Internal(e.into()))?;
        Ok(())
    }

    async fn update_backup_codes(&self, id: Uuid, backup_codes_enc: &str) -> Result<(), AppError> {
        sqlx::query(
            "UPDATE mfa_enrollments SET backup_codes_enc = $2, updated_at = now() WHERE id = $1",
        )
        .bind(id)
        .bind(backup_codes_enc)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;
        Ok(())
    }

    async fn update_last_used(&self, id: Uuid) -> Result<(), AppError> {
        sqlx::query(
            "UPDATE mfa_enrollments SET last_used_at = now(), updated_at = now() WHERE id = $1",
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;
        Ok(())
    }

    async fn delete(&self, id: Uuid) -> Result<(), AppError> {
        sqlx::query("DELETE FROM mfa_enrollments WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Internal(e.into()))?;
        Ok(())
    }
}
