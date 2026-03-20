use async_trait::async_trait;
use chrono::{DateTime, Utc};
use nucleus_core::error::AppError;
use nucleus_core::pagination::{PaginatedResponse, PaginationParams};
use nucleus_core::types::{AccountId, ProjectId};
use serde::Serialize;
use sqlx::postgres::PgRow;
use sqlx::{PgPool, Row};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
pub struct Project {
    pub id: ProjectId,
    pub account_id: AccountId,
    pub name: String,
    pub slug: String,
    pub data_mode: String,
    pub environment: String,
    pub plan_id: Uuid,
    pub webhook_url: Option<String>,
    pub webhook_secret: Option<String>,
    pub allowed_origins: Vec<String>,
    pub session_ttl: i32,
    pub jwt_lifetime: i32,
    pub jwt_algorithm: String,
    pub settings: serde_json::Value,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct NewProject {
    pub account_id: AccountId,
    pub name: String,
    pub slug: String,
    pub plan_id: Uuid,
    pub data_mode: Option<String>,
}

fn project_from_row(row: &PgRow) -> Result<Project, sqlx::Error> {
    Ok(Project {
        id: ProjectId::from_uuid(row.try_get("id")?),
        account_id: AccountId::from_uuid(row.try_get("account_id")?),
        name: row.try_get("name")?,
        slug: row.try_get("slug")?,
        data_mode: row.try_get::<String, _>("data_mode")?,
        environment: row.try_get::<String, _>("environment")?,
        plan_id: row.try_get("plan_id")?,
        webhook_url: row.try_get("webhook_url")?,
        webhook_secret: row.try_get("webhook_secret")?,
        allowed_origins: row.try_get("allowed_origins")?,
        session_ttl: row.try_get("session_ttl")?,
        jwt_lifetime: row.try_get("jwt_lifetime")?,
        jwt_algorithm: row.try_get("jwt_algorithm")?,
        settings: row.try_get("settings")?,
        is_active: row.try_get("is_active")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

#[async_trait]
pub trait ProjectRepository: Send + Sync {
    async fn create(&self, project: &NewProject) -> Result<Project, AppError>;
    async fn find_by_id(&self, id: &ProjectId) -> Result<Option<Project>, AppError>;
    async fn find_by_slug(&self, slug: &str) -> Result<Option<Project>, AppError>;
    async fn list_by_account(
        &self,
        account_id: &AccountId,
        params: &PaginationParams,
    ) -> Result<PaginatedResponse<Project>, AppError>;
    async fn update_settings(
        &self,
        id: &ProjectId,
        settings: serde_json::Value,
    ) -> Result<Project, AppError>;
}

pub struct PgProjectRepository {
    pool: PgPool,
}

impl PgProjectRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ProjectRepository for PgProjectRepository {
    async fn create(&self, project: &NewProject) -> Result<Project, AppError> {
        let data_mode = project.data_mode.as_deref().unwrap_or("centralized");

        let row = sqlx::query(
            r#"
            INSERT INTO projects (account_id, name, slug, plan_id, data_mode)
            VALUES ($1, $2, $3, $4, $5::data_mode)
            RETURNING *
            "#,
        )
        .bind(project.account_id.0)
        .bind(&project.name)
        .bind(&project.slug)
        .bind(project.plan_id)
        .bind(data_mode)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

        project_from_row(&row).map_err(|e| AppError::Internal(e.into()))
    }

    async fn find_by_id(&self, id: &ProjectId) -> Result<Option<Project>, AppError> {
        let row = sqlx::query("SELECT * FROM projects WHERE id = $1")
            .bind(id.0)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AppError::Internal(e.into()))?;

        match row {
            Some(ref r) => Ok(Some(
                project_from_row(r).map_err(|e| AppError::Internal(e.into()))?,
            )),
            None => Ok(None),
        }
    }

    async fn find_by_slug(&self, slug: &str) -> Result<Option<Project>, AppError> {
        let row = sqlx::query("SELECT * FROM projects WHERE slug = $1")
            .bind(slug)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AppError::Internal(e.into()))?;

        match row {
            Some(ref r) => Ok(Some(
                project_from_row(r).map_err(|e| AppError::Internal(e.into()))?,
            )),
            None => Ok(None),
        }
    }

    async fn list_by_account(
        &self,
        account_id: &AccountId,
        params: &PaginationParams,
    ) -> Result<PaginatedResponse<Project>, AppError> {
        let limit = params.effective_limit() as i64;
        let fetch_limit = limit + 1;

        let rows = if let Some(ref cursor) = params.cursor {
            let cursor_id: Uuid = cursor
                .parse()
                .map_err(|e: uuid::Error| AppError::Internal(e.into()))?;
            sqlx::query(
                r#"
                SELECT * FROM projects
                WHERE account_id = $1 AND id > $2
                ORDER BY id ASC
                LIMIT $3
                "#,
            )
            .bind(account_id.0)
            .bind(cursor_id)
            .bind(fetch_limit)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AppError::Internal(e.into()))?
        } else {
            sqlx::query(
                r#"
                SELECT * FROM projects
                WHERE account_id = $1
                ORDER BY id ASC
                LIMIT $2
                "#,
            )
            .bind(account_id.0)
            .bind(fetch_limit)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AppError::Internal(e.into()))?
        };

        let has_more = rows.len() as i64 > limit;
        let items: Vec<Project> = rows
            .iter()
            .take(limit as usize)
            .map(|r| project_from_row(r))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| AppError::Internal(e.into()))?;

        let next_cursor = if has_more {
            items.last().map(|p| p.id.0.to_string())
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

    async fn update_settings(
        &self,
        id: &ProjectId,
        settings: serde_json::Value,
    ) -> Result<Project, AppError> {
        let row = sqlx::query(
            r#"
            UPDATE projects SET settings = $1, updated_at = now()
            WHERE id = $2
            RETURNING *
            "#,
        )
        .bind(&settings)
        .bind(id.0)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

        project_from_row(&row).map_err(|e| AppError::Internal(e.into()))
    }
}
