use async_trait::async_trait;
use chrono::{DateTime, Utc};
use nucleus_core::error::AppError;
use nucleus_core::pagination::{PaginatedResponse, PaginationParams};
use nucleus_core::types::{ProjectId, UserId};
use serde::Serialize;
use sqlx::postgres::PgRow;
use sqlx::{PgPool, Row};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
pub struct AuditLog {
    pub id: Uuid,
    pub project_id: ProjectId,
    pub actor_type: String,
    pub actor_id: Option<Uuid>,
    pub action: String,
    pub target_type: Option<String>,
    pub target_id: Option<Uuid>,
    pub metadata: serde_json::Value,
    pub ip: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: DateTime<Utc>,
}

pub struct NewAuditLog {
    pub project_id: ProjectId,
    pub actor_type: String,
    pub actor_id: Option<Uuid>,
    pub action: String,
    pub target_type: Option<String>,
    pub target_id: Option<Uuid>,
    pub metadata: Option<serde_json::Value>,
    pub ip: Option<String>,
    pub user_agent: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SignInAttempt {
    pub id: Uuid,
    pub project_id: ProjectId,
    pub user_id: Option<UserId>,
    pub method: String,
    pub status: String,
    pub failure_reason: Option<String>,
    pub ip: Option<String>,
    pub user_agent: Option<String>,
    pub country_code: Option<String>,
    pub city: Option<String>,
    pub created_at: DateTime<Utc>,
}

pub struct NewSignInAttempt {
    pub project_id: ProjectId,
    pub user_id: Option<UserId>,
    pub method: String,
    pub status: String,
    pub failure_reason: Option<String>,
    pub ip: Option<String>,
    pub user_agent: Option<String>,
    pub country_code: Option<String>,
    pub city: Option<String>,
}

fn audit_log_from_row(row: &PgRow) -> Result<AuditLog, sqlx::Error> {
    Ok(AuditLog {
        id: row.try_get("id")?,
        project_id: ProjectId::from_uuid(row.try_get("project_id")?),
        actor_type: row.try_get("actor_type")?,
        actor_id: row.try_get("actor_id")?,
        action: row.try_get("action")?,
        target_type: row.try_get("target_type")?,
        target_id: row.try_get("target_id")?,
        metadata: row.try_get("metadata")?,
        ip: row
            .try_get::<Option<String>, _>("ip")
            .or_else(|_: sqlx::Error| Ok::<Option<String>, sqlx::Error>(None))?,
        user_agent: row.try_get("user_agent")?,
        created_at: row.try_get("created_at")?,
    })
}

fn sign_in_attempt_from_row(row: &PgRow) -> Result<SignInAttempt, sqlx::Error> {
    Ok(SignInAttempt {
        id: row.try_get("id")?,
        project_id: ProjectId::from_uuid(row.try_get("project_id")?),
        user_id: row
            .try_get::<Option<Uuid>, _>("user_id")?
            .map(UserId::from_uuid),
        method: row.try_get("method")?,
        status: row.try_get("status")?,
        failure_reason: row.try_get("failure_reason")?,
        ip: row
            .try_get::<Option<String>, _>("ip")
            .or_else(|_: sqlx::Error| Ok::<Option<String>, sqlx::Error>(None))?,
        user_agent: row.try_get("user_agent")?,
        country_code: row.try_get("country_code")?,
        city: row.try_get("city")?,
        created_at: row.try_get("created_at")?,
    })
}

#[async_trait]
pub trait AuditRepository: Send + Sync {
    async fn create_audit_log(&self, log: &NewAuditLog) -> Result<AuditLog, AppError>;
    async fn list_audit_logs(
        &self,
        project_id: &ProjectId,
        params: &PaginationParams,
    ) -> Result<PaginatedResponse<AuditLog>, AppError>;
    async fn create_sign_in_attempt(
        &self,
        attempt: &NewSignInAttempt,
    ) -> Result<SignInAttempt, AppError>;
    async fn list_sign_in_attempts(
        &self,
        project_id: &ProjectId,
        user_id: &UserId,
        params: &PaginationParams,
    ) -> Result<PaginatedResponse<SignInAttempt>, AppError>;
}

pub struct PgAuditRepository {
    pool: PgPool,
}

impl PgAuditRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AuditRepository for PgAuditRepository {
    async fn create_audit_log(&self, log: &NewAuditLog) -> Result<AuditLog, AppError> {
        let metadata = log
            .metadata
            .as_ref()
            .cloned()
            .unwrap_or(serde_json::json!({}));

        let row = sqlx::query(
            r#"
            INSERT INTO audit_logs (project_id, actor_type, actor_id, action, target_type, target_id, metadata, ip, user_agent)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8::inet, $9)
            RETURNING *
            "#,
        )
        .bind(log.project_id.0)
        .bind(&log.actor_type)
        .bind(log.actor_id)
        .bind(&log.action)
        .bind(&log.target_type)
        .bind(log.target_id)
        .bind(&metadata)
        .bind(&log.ip)
        .bind(&log.user_agent)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

        audit_log_from_row(&row).map_err(|e| AppError::Internal(e.into()))
    }

    async fn list_audit_logs(
        &self,
        project_id: &ProjectId,
        params: &PaginationParams,
    ) -> Result<PaginatedResponse<AuditLog>, AppError> {
        let limit = params.effective_limit() as i64;
        let fetch_limit = limit + 1;

        let rows = if let Some(ref cursor) = params.cursor {
            let cursor_id: Uuid = cursor
                .parse()
                .map_err(|e: uuid::Error| AppError::Internal(e.into()))?;
            sqlx::query(
                r#"
                SELECT * FROM audit_logs
                WHERE project_id = $1 AND id < $2
                ORDER BY created_at DESC, id DESC
                LIMIT $3
                "#,
            )
            .bind(project_id.0)
            .bind(cursor_id)
            .bind(fetch_limit)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AppError::Internal(e.into()))?
        } else {
            sqlx::query(
                r#"
                SELECT * FROM audit_logs
                WHERE project_id = $1
                ORDER BY created_at DESC, id DESC
                LIMIT $2
                "#,
            )
            .bind(project_id.0)
            .bind(fetch_limit)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AppError::Internal(e.into()))?
        };

        let has_more = rows.len() as i64 > limit;
        let items: Vec<AuditLog> = rows
            .iter()
            .take(limit as usize)
            .map(|r| audit_log_from_row(r))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| AppError::Internal(e.into()))?;

        let next_cursor = if has_more {
            items.last().map(|a| a.id.to_string())
        } else {
            None
        };

        Ok(PaginatedResponse {
            data: items,
            has_more,
            next_cursor,
            total_count: None,
        })
    }

    async fn create_sign_in_attempt(
        &self,
        attempt: &NewSignInAttempt,
    ) -> Result<SignInAttempt, AppError> {
        let row = sqlx::query(
            r#"
            INSERT INTO sign_in_attempts (project_id, user_id, method, status, failure_reason, ip, user_agent, country_code, city)
            VALUES ($1, $2, $3, $4, $5, $6::inet, $7, $8, $9)
            RETURNING *
            "#,
        )
        .bind(attempt.project_id.0)
        .bind(attempt.user_id.map(|u| u.0))
        .bind(&attempt.method)
        .bind(&attempt.status)
        .bind(&attempt.failure_reason)
        .bind(&attempt.ip)
        .bind(&attempt.user_agent)
        .bind(&attempt.country_code)
        .bind(&attempt.city)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

        sign_in_attempt_from_row(&row).map_err(|e| AppError::Internal(e.into()))
    }

    async fn list_sign_in_attempts(
        &self,
        project_id: &ProjectId,
        user_id: &UserId,
        params: &PaginationParams,
    ) -> Result<PaginatedResponse<SignInAttempt>, AppError> {
        let limit = params.effective_limit() as i64;
        let fetch_limit = limit + 1;

        let rows = if let Some(ref cursor) = params.cursor {
            let cursor_id: Uuid = cursor
                .parse()
                .map_err(|e: uuid::Error| AppError::Internal(e.into()))?;
            sqlx::query(
                r#"
                SELECT * FROM sign_in_attempts
                WHERE project_id = $1 AND user_id = $2 AND id < $3
                ORDER BY created_at DESC, id DESC
                LIMIT $4
                "#,
            )
            .bind(project_id.0)
            .bind(user_id.0)
            .bind(cursor_id)
            .bind(fetch_limit)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AppError::Internal(e.into()))?
        } else {
            sqlx::query(
                r#"
                SELECT * FROM sign_in_attempts
                WHERE project_id = $1 AND user_id = $2
                ORDER BY created_at DESC, id DESC
                LIMIT $3
                "#,
            )
            .bind(project_id.0)
            .bind(user_id.0)
            .bind(fetch_limit)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AppError::Internal(e.into()))?
        };

        let has_more = rows.len() as i64 > limit;
        let items: Vec<SignInAttempt> = rows
            .iter()
            .take(limit as usize)
            .map(|r| sign_in_attempt_from_row(r))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| AppError::Internal(e.into()))?;

        let next_cursor = if has_more {
            items.last().map(|a| a.id.to_string())
        } else {
            None
        };

        Ok(PaginatedResponse {
            data: items,
            has_more,
            next_cursor,
            total_count: None,
        })
    }
}
