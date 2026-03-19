use async_trait::async_trait;
use chrono::{DateTime, Utc};
use nucleus_core::error::AppError;
use nucleus_core::types::ProjectId;
use serde::Serialize;
use sqlx::postgres::PgRow;
use sqlx::{PgPool, Row};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
pub struct WebhookEvent {
    pub id: Uuid,
    pub project_id: ProjectId,
    pub event_type: String,
    pub payload: serde_json::Value,
    pub status: String,
    pub attempts: i32,
    pub max_attempts: i32,
    pub last_attempt_at: Option<DateTime<Utc>>,
    pub last_response_code: Option<i32>,
    pub last_error: Option<String>,
    pub next_retry_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

pub struct NewWebhookEvent {
    pub project_id: ProjectId,
    pub event_type: String,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, Serialize)]
pub struct WebhookDeliveryLog {
    pub event_id: Uuid,
    pub attempt_number: i32,
    pub response_code: Option<i32>,
    pub response_body: Option<String>,
    pub latency_ms: Option<i32>,
    pub error: Option<String>,
}

fn webhook_event_from_row(row: &PgRow) -> Result<WebhookEvent, sqlx::Error> {
    Ok(WebhookEvent {
        id: row.try_get("id")?,
        project_id: ProjectId::from_uuid(row.try_get("project_id")?),
        event_type: row.try_get("event_type")?,
        payload: row.try_get("payload")?,
        status: row.try_get("status")?,
        attempts: row.try_get("attempts")?,
        max_attempts: row.try_get("max_attempts")?,
        last_attempt_at: row.try_get("last_attempt_at")?,
        last_response_code: row.try_get("last_response_code")?,
        last_error: row.try_get("last_error")?,
        next_retry_at: row.try_get("next_retry_at")?,
        created_at: row.try_get("created_at")?,
    })
}

#[async_trait]
pub trait WebhookRepository: Send + Sync {
    async fn create_event(&self, event: &NewWebhookEvent) -> Result<WebhookEvent, AppError>;
    async fn find_pending_events(&self, limit: u32) -> Result<Vec<WebhookEvent>, AppError>;
    async fn update_event_status(
        &self,
        id: Uuid,
        status: &str,
        response_code: Option<i32>,
        error: Option<&str>,
    ) -> Result<(), AppError>;
    async fn create_delivery_log(&self, log: &WebhookDeliveryLog) -> Result<(), AppError>;
}

pub struct PgWebhookRepository {
    pool: PgPool,
}

impl PgWebhookRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl WebhookRepository for PgWebhookRepository {
    async fn create_event(&self, event: &NewWebhookEvent) -> Result<WebhookEvent, AppError> {
        let row = sqlx::query(
            r#"
            INSERT INTO webhook_events (project_id, event_type, payload)
            VALUES ($1, $2, $3)
            RETURNING *
            "#,
        )
        .bind(event.project_id.0)
        .bind(&event.event_type)
        .bind(&event.payload)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

        webhook_event_from_row(&row).map_err(|e| AppError::Internal(e.into()))
    }

    async fn find_pending_events(&self, limit: u32) -> Result<Vec<WebhookEvent>, AppError> {
        let rows = sqlx::query(
            r#"
            SELECT * FROM webhook_events
            WHERE status = 'pending'
              AND attempts < max_attempts
              AND (next_retry_at IS NULL OR next_retry_at <= now())
            ORDER BY created_at ASC
            LIMIT $1
            "#,
        )
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

        rows.iter()
            .map(|r| webhook_event_from_row(r))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| AppError::Internal(e.into()))
    }

    async fn update_event_status(
        &self,
        id: Uuid,
        status: &str,
        response_code: Option<i32>,
        error: Option<&str>,
    ) -> Result<(), AppError> {
        sqlx::query(
            r#"
            UPDATE webhook_events SET
                status = $2,
                attempts = attempts + 1,
                last_attempt_at = now(),
                last_response_code = $3,
                last_error = $4,
                next_retry_at = CASE
                    WHEN $2 = 'pending' THEN now() + (interval '1 second' * power(2, attempts + 1))
                    ELSE NULL
                END
            WHERE id = $1
            "#,
        )
        .bind(id)
        .bind(status)
        .bind(response_code)
        .bind(error)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

        Ok(())
    }

    async fn create_delivery_log(&self, log: &WebhookDeliveryLog) -> Result<(), AppError> {
        sqlx::query(
            r#"
            INSERT INTO webhook_delivery_logs (event_id, attempt_number, response_code, response_body, latency_ms, error)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(log.event_id)
        .bind(log.attempt_number)
        .bind(log.response_code)
        .bind(&log.response_body)
        .bind(log.latency_ms)
        .bind(&log.error)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

        Ok(())
    }
}
