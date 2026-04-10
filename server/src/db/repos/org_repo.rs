use crate::core::error::AppError;
use crate::core::pagination::{PaginatedResponse, PaginationParams};
use crate::core::types::{OrgId, ProjectId, RoleId, UserId};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::postgres::PgRow;
use sqlx::{PgPool, Row};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
pub struct Organization {
    pub id: OrgId,
    pub project_id: ProjectId,
    pub name: String,
    pub slug: String,
    pub logo_url: Option<String>,
    pub metadata: serde_json::Value,
    pub max_members: Option<i32>,
    pub created_by: Option<UserId>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct NewOrganization {
    pub project_id: ProjectId,
    pub name: String,
    pub slug: String,
    pub logo_url: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub max_members: Option<i32>,
    pub created_by: Option<UserId>,
}

#[derive(Debug, Clone, Serialize)]
pub struct OrgMember {
    pub id: Uuid,
    pub org_id: OrgId,
    pub user_id: UserId,
    pub role_id: RoleId,
    pub invited_by: Option<UserId>,
    pub joined_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

pub struct NewOrgMember {
    pub org_id: OrgId,
    pub user_id: UserId,
    pub role_id: RoleId,
    pub invited_by: Option<UserId>,
}

fn org_from_row(row: &PgRow) -> Result<Organization, sqlx::Error> {
    Ok(Organization {
        id: OrgId::from_uuid(row.try_get("id")?),
        project_id: ProjectId::from_uuid(row.try_get("project_id")?),
        name: row.try_get("name")?,
        slug: row.try_get("slug")?,
        logo_url: row.try_get("logo_url")?,
        metadata: row.try_get("metadata")?,
        max_members: row.try_get("max_members")?,
        created_by: row
            .try_get::<Option<Uuid>, _>("created_by")?
            .map(UserId::from_uuid),
        deleted_at: row.try_get("deleted_at")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

fn org_member_from_row(row: &PgRow) -> Result<OrgMember, sqlx::Error> {
    Ok(OrgMember {
        id: row.try_get("id")?,
        org_id: OrgId::from_uuid(row.try_get("org_id")?),
        user_id: UserId::from_uuid(row.try_get("user_id")?),
        role_id: RoleId::from_uuid(row.try_get("role_id")?),
        invited_by: row
            .try_get::<Option<Uuid>, _>("invited_by")?
            .map(UserId::from_uuid),
        joined_at: row.try_get("joined_at")?,
        created_at: row.try_get("created_at")?,
    })
}

#[async_trait]
pub trait OrgRepository: Send + Sync {
    async fn create(&self, org: &NewOrganization) -> Result<Organization, AppError>;
    async fn find_by_id(
        &self,
        project_id: &ProjectId,
        org_id: &OrgId,
    ) -> Result<Option<Organization>, AppError>;
    async fn find_by_slug(
        &self,
        project_id: &ProjectId,
        slug: &str,
    ) -> Result<Option<Organization>, AppError>;
    async fn list_by_project(
        &self,
        project_id: &ProjectId,
        params: &PaginationParams,
    ) -> Result<PaginatedResponse<Organization>, AppError>;
    async fn list_by_user(
        &self,
        project_id: &ProjectId,
        user_id: &UserId,
    ) -> Result<Vec<Organization>, AppError>;
    async fn add_member(&self, member: &NewOrgMember) -> Result<OrgMember, AppError>;
    async fn remove_member(&self, org_id: &OrgId, user_id: &UserId) -> Result<(), AppError>;
    async fn update_member_role(
        &self,
        org_id: &OrgId,
        user_id: &UserId,
        role_id: &RoleId,
    ) -> Result<(), AppError>;
    async fn list_members(
        &self,
        org_id: &OrgId,
        params: &PaginationParams,
    ) -> Result<PaginatedResponse<OrgMember>, AppError>;
}

pub struct PgOrgRepository {
    pool: PgPool,
}

impl PgOrgRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl OrgRepository for PgOrgRepository {
    async fn create(&self, org: &NewOrganization) -> Result<Organization, AppError> {
        let metadata = org
            .metadata
            .as_ref()
            .cloned()
            .unwrap_or(serde_json::json!({}));

        let row = sqlx::query(
            r#"
            INSERT INTO organizations (project_id, name, slug, logo_url, metadata, max_members, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING *
            "#,
        )
        .bind(org.project_id.0)
        .bind(&org.name)
        .bind(&org.slug)
        .bind(&org.logo_url)
        .bind(&metadata)
        .bind(org.max_members)
        .bind(org.created_by.map(|u| u.0))
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

        org_from_row(&row).map_err(|e| AppError::Internal(e.into()))
    }

    async fn find_by_id(
        &self,
        project_id: &ProjectId,
        org_id: &OrgId,
    ) -> Result<Option<Organization>, AppError> {
        let row = sqlx::query(
            "SELECT * FROM organizations WHERE project_id = $1 AND id = $2 AND deleted_at IS NULL",
        )
        .bind(project_id.0)
        .bind(org_id.0)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

        match row {
            Some(ref r) => Ok(Some(
                org_from_row(r).map_err(|e| AppError::Internal(e.into()))?,
            )),
            None => Ok(None),
        }
    }

    async fn find_by_slug(
        &self,
        project_id: &ProjectId,
        slug: &str,
    ) -> Result<Option<Organization>, AppError> {
        let row = sqlx::query(
            "SELECT * FROM organizations WHERE project_id = $1 AND slug = $2 AND deleted_at IS NULL",
        )
        .bind(project_id.0)
        .bind(slug)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

        match row {
            Some(ref r) => Ok(Some(
                org_from_row(r).map_err(|e| AppError::Internal(e.into()))?,
            )),
            None => Ok(None),
        }
    }

    async fn list_by_project(
        &self,
        project_id: &ProjectId,
        params: &PaginationParams,
    ) -> Result<PaginatedResponse<Organization>, AppError> {
        let limit = params.effective_limit() as i64;
        let fetch_limit = limit + 1;

        let rows = if let Some(ref cursor) = params.cursor {
            let cursor_id: Uuid = cursor
                .parse()
                .map_err(|e: uuid::Error| AppError::Internal(e.into()))?;
            sqlx::query(
                r#"
                SELECT * FROM organizations
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
                SELECT * FROM organizations
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
        let items: Vec<Organization> = rows
            .iter()
            .take(limit as usize)
            .map(org_from_row)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| AppError::Internal(e.into()))?;

        let next_cursor = if has_more {
            items.last().map(|o| o.id.0.to_string())
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

    async fn list_by_user(
        &self,
        project_id: &ProjectId,
        user_id: &UserId,
    ) -> Result<Vec<Organization>, AppError> {
        let rows = sqlx::query(
            r#"
            SELECT o.* FROM organizations o
            INNER JOIN org_members m ON m.org_id = o.id
            WHERE o.project_id = $1 AND m.user_id = $2 AND o.deleted_at IS NULL
            ORDER BY o.name ASC
            "#,
        )
        .bind(project_id.0)
        .bind(user_id.0)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

        rows.iter()
            .map(org_from_row)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| AppError::Internal(e.into()))
    }

    async fn add_member(&self, member: &NewOrgMember) -> Result<OrgMember, AppError> {
        let row = sqlx::query(
            r#"
            INSERT INTO org_members (org_id, user_id, role_id, invited_by, joined_at)
            VALUES ($1, $2, $3, $4, now())
            RETURNING *
            "#,
        )
        .bind(member.org_id.0)
        .bind(member.user_id.0)
        .bind(member.role_id.0)
        .bind(member.invited_by.map(|u| u.0))
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

        org_member_from_row(&row).map_err(|e| AppError::Internal(e.into()))
    }

    async fn remove_member(&self, org_id: &OrgId, user_id: &UserId) -> Result<(), AppError> {
        sqlx::query("DELETE FROM org_members WHERE org_id = $1 AND user_id = $2")
            .bind(org_id.0)
            .bind(user_id.0)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Internal(e.into()))?;

        Ok(())
    }

    async fn update_member_role(
        &self,
        org_id: &OrgId,
        user_id: &UserId,
        role_id: &RoleId,
    ) -> Result<(), AppError> {
        sqlx::query("UPDATE org_members SET role_id = $3 WHERE org_id = $1 AND user_id = $2")
            .bind(org_id.0)
            .bind(user_id.0)
            .bind(role_id.0)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Internal(e.into()))?;

        Ok(())
    }

    async fn list_members(
        &self,
        org_id: &OrgId,
        params: &PaginationParams,
    ) -> Result<PaginatedResponse<OrgMember>, AppError> {
        let limit = params.effective_limit() as i64;
        let fetch_limit = limit + 1;

        let rows = if let Some(ref cursor) = params.cursor {
            let cursor_id: Uuid = cursor
                .parse()
                .map_err(|e: uuid::Error| AppError::Internal(e.into()))?;
            sqlx::query(
                r#"
                SELECT * FROM org_members
                WHERE org_id = $1 AND id > $2
                ORDER BY id ASC
                LIMIT $3
                "#,
            )
            .bind(org_id.0)
            .bind(cursor_id)
            .bind(fetch_limit)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AppError::Internal(e.into()))?
        } else {
            sqlx::query(
                r#"
                SELECT * FROM org_members
                WHERE org_id = $1
                ORDER BY id ASC
                LIMIT $2
                "#,
            )
            .bind(org_id.0)
            .bind(fetch_limit)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AppError::Internal(e.into()))?
        };

        let has_more = rows.len() as i64 > limit;
        let items: Vec<OrgMember> = rows
            .iter()
            .take(limit as usize)
            .map(org_member_from_row)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| AppError::Internal(e.into()))?;

        let next_cursor = if has_more {
            items.last().map(|m| m.id.to_string())
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
