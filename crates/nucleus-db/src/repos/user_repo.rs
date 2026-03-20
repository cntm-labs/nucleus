use async_trait::async_trait;
use chrono::{DateTime, Utc};
use nucleus_core::error::AppError;
use nucleus_core::pagination::{PaginatedResponse, PaginationParams};
use nucleus_core::types::{ProjectId, UserId};
use serde::Serialize;
use sqlx::postgres::PgRow;
use sqlx::{PgPool, Row};

#[derive(Debug, Clone, Serialize)]
pub struct User {
    pub id: UserId,
    pub project_id: ProjectId,
    pub external_id: Option<String>,
    pub email: String,
    pub email_verified: bool,
    pub phone: Option<String>,
    pub phone_verified: bool,
    pub username: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub avatar_url: Option<String>,
    pub metadata: serde_json::Value,
    pub private_metadata: serde_json::Value,
    pub last_sign_in_at: Option<DateTime<Utc>>,
    pub banned_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct NewUser {
    pub email: String,
    pub username: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub external_id: Option<String>,
    pub phone: Option<String>,
    pub avatar_url: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

pub struct UpdateUser {
    pub email: Option<String>,
    pub username: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub avatar_url: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub private_metadata: Option<serde_json::Value>,
}

fn user_from_row(row: &PgRow) -> Result<User, sqlx::Error> {
    Ok(User {
        id: UserId::from_uuid(row.try_get("id")?),
        project_id: ProjectId::from_uuid(row.try_get("project_id")?),
        external_id: row.try_get("external_id")?,
        email: row.try_get("email")?,
        email_verified: row.try_get("email_verified")?,
        phone: row.try_get("phone")?,
        phone_verified: row.try_get("phone_verified")?,
        username: row.try_get("username")?,
        first_name: row.try_get("first_name")?,
        last_name: row.try_get("last_name")?,
        avatar_url: row.try_get("avatar_url")?,
        metadata: row.try_get("metadata")?,
        private_metadata: row.try_get("private_metadata")?,
        last_sign_in_at: row.try_get("last_sign_in_at")?,
        banned_at: row.try_get("banned_at")?,
        deleted_at: row.try_get("deleted_at")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn create(&self, project_id: &ProjectId, user: &NewUser) -> Result<User, AppError>;
    async fn find_by_id(
        &self,
        project_id: &ProjectId,
        user_id: &UserId,
    ) -> Result<Option<User>, AppError>;
    async fn find_by_email(
        &self,
        project_id: &ProjectId,
        email: &str,
    ) -> Result<Option<User>, AppError>;
    async fn find_by_username(
        &self,
        project_id: &ProjectId,
        username: &str,
    ) -> Result<Option<User>, AppError>;
    async fn update(
        &self,
        project_id: &ProjectId,
        user_id: &UserId,
        update: &UpdateUser,
    ) -> Result<User, AppError>;
    async fn soft_delete(&self, project_id: &ProjectId, user_id: &UserId) -> Result<(), AppError>;
    async fn list(
        &self,
        project_id: &ProjectId,
        params: &PaginationParams,
    ) -> Result<PaginatedResponse<User>, AppError>;
    async fn ban(&self, project_id: &ProjectId, user_id: &UserId) -> Result<(), AppError>;
    async fn unban(&self, project_id: &ProjectId, user_id: &UserId) -> Result<(), AppError>;
}

pub struct PgUserRepository {
    pool: PgPool,
}

impl PgUserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepository for PgUserRepository {
    async fn create(&self, project_id: &ProjectId, user: &NewUser) -> Result<User, AppError> {
        let metadata = user
            .metadata
            .as_ref()
            .cloned()
            .unwrap_or(serde_json::json!({}));

        let row = sqlx::query(
            r#"
            INSERT INTO users (project_id, email, username, first_name, last_name, external_id, phone, avatar_url, metadata)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING *
            "#,
        )
        .bind(project_id.0)
        .bind(&user.email)
        .bind(&user.username)
        .bind(&user.first_name)
        .bind(&user.last_name)
        .bind(&user.external_id)
        .bind(&user.phone)
        .bind(&user.avatar_url)
        .bind(&metadata)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

        user_from_row(&row).map_err(|e| AppError::Internal(e.into()))
    }

    async fn find_by_id(
        &self,
        project_id: &ProjectId,
        user_id: &UserId,
    ) -> Result<Option<User>, AppError> {
        let row = sqlx::query(
            "SELECT * FROM users WHERE project_id = $1 AND id = $2 AND deleted_at IS NULL",
        )
        .bind(project_id.0)
        .bind(user_id.0)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

        match row {
            Some(ref r) => Ok(Some(
                user_from_row(r).map_err(|e| AppError::Internal(e.into()))?,
            )),
            None => Ok(None),
        }
    }

    async fn find_by_email(
        &self,
        project_id: &ProjectId,
        email: &str,
    ) -> Result<Option<User>, AppError> {
        let row = sqlx::query(
            "SELECT * FROM users WHERE project_id = $1 AND email = $2 AND deleted_at IS NULL",
        )
        .bind(project_id.0)
        .bind(email)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

        match row {
            Some(ref r) => Ok(Some(
                user_from_row(r).map_err(|e| AppError::Internal(e.into()))?,
            )),
            None => Ok(None),
        }
    }

    async fn find_by_username(
        &self,
        project_id: &ProjectId,
        username: &str,
    ) -> Result<Option<User>, AppError> {
        let row = sqlx::query(
            "SELECT * FROM users WHERE project_id = $1 AND username = $2 AND deleted_at IS NULL",
        )
        .bind(project_id.0)
        .bind(username)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

        match row {
            Some(ref r) => Ok(Some(
                user_from_row(r).map_err(|e| AppError::Internal(e.into()))?,
            )),
            None => Ok(None),
        }
    }

    async fn update(
        &self,
        project_id: &ProjectId,
        user_id: &UserId,
        update: &UpdateUser,
    ) -> Result<User, AppError> {
        let row = sqlx::query(
            r#"
            UPDATE users SET
                email = COALESCE($3, email),
                username = COALESCE($4, username),
                first_name = COALESCE($5, first_name),
                last_name = COALESCE($6, last_name),
                avatar_url = COALESCE($7, avatar_url),
                metadata = COALESCE($8, metadata),
                private_metadata = COALESCE($9, private_metadata),
                updated_at = now()
            WHERE project_id = $1 AND id = $2 AND deleted_at IS NULL
            RETURNING *
            "#,
        )
        .bind(project_id.0)
        .bind(user_id.0)
        .bind(&update.email)
        .bind(&update.username)
        .bind(&update.first_name)
        .bind(&update.last_name)
        .bind(&update.avatar_url)
        .bind(&update.metadata)
        .bind(&update.private_metadata)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

        user_from_row(&row).map_err(|e| AppError::Internal(e.into()))
    }

    async fn soft_delete(&self, project_id: &ProjectId, user_id: &UserId) -> Result<(), AppError> {
        sqlx::query(
            "UPDATE users SET deleted_at = now(), updated_at = now() WHERE project_id = $1 AND id = $2 AND deleted_at IS NULL",
        )
        .bind(project_id.0)
        .bind(user_id.0)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

        Ok(())
    }

    async fn list(
        &self,
        project_id: &ProjectId,
        params: &PaginationParams,
    ) -> Result<PaginatedResponse<User>, AppError> {
        let limit = params.effective_limit() as i64;
        let fetch_limit = limit + 1;

        let rows = if let Some(ref cursor) = params.cursor {
            let cursor_id: uuid::Uuid = cursor
                .parse()
                .map_err(|e: uuid::Error| AppError::Internal(e.into()))?;
            sqlx::query(
                r#"
                SELECT * FROM users
                WHERE project_id = $1 AND deleted_at IS NULL AND id > $2
                ORDER BY id ASC
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
                SELECT * FROM users
                WHERE project_id = $1 AND deleted_at IS NULL
                ORDER BY id ASC
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
        let items: Vec<User> = rows
            .iter()
            .take(limit as usize)
            .map(user_from_row)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| AppError::Internal(e.into()))?;

        let next_cursor = if has_more {
            items.last().map(|u| u.id.0.to_string())
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

    async fn ban(&self, project_id: &ProjectId, user_id: &UserId) -> Result<(), AppError> {
        sqlx::query(
            "UPDATE users SET banned_at = now(), updated_at = now() WHERE project_id = $1 AND id = $2 AND deleted_at IS NULL",
        )
        .bind(project_id.0)
        .bind(user_id.0)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

        Ok(())
    }

    async fn unban(&self, project_id: &ProjectId, user_id: &UserId) -> Result<(), AppError> {
        sqlx::query(
            "UPDATE users SET banned_at = NULL, updated_at = now() WHERE project_id = $1 AND id = $2 AND deleted_at IS NULL",
        )
        .bind(project_id.0)
        .bind(user_id.0)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

        Ok(())
    }
}
