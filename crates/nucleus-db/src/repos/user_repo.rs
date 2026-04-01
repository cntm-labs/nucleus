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

/// In-memory mock for testing UserRepository trait contracts.
/// Used to verify expected behavior without a real database.
#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use chrono::Utc;
    use std::sync::Mutex;

    struct MockUserRepo {
        users: Mutex<Vec<User>>,
    }

    impl MockUserRepo {
        fn new() -> Self {
            Self {
                users: Mutex::new(Vec::new()),
            }
        }
    }

    #[async_trait]
    impl UserRepository for MockUserRepo {
        async fn create(&self, project_id: &ProjectId, new: &NewUser) -> Result<User, AppError> {
            let user = User {
                id: UserId::new(),
                project_id: *project_id,
                external_id: new.external_id.clone(),
                email: new.email.clone(),
                email_verified: false,
                phone: new.phone.clone(),
                phone_verified: false,
                username: new.username.clone(),
                first_name: new.first_name.clone(),
                last_name: new.last_name.clone(),
                avatar_url: new.avatar_url.clone(),
                metadata: new
                    .metadata
                    .clone()
                    .unwrap_or(serde_json::Value::Object(Default::default())),
                private_metadata: serde_json::Value::Object(Default::default()),
                last_sign_in_at: None,
                banned_at: None,
                deleted_at: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };
            self.users.lock().unwrap().push(user.clone());
            Ok(user)
        }

        async fn find_by_id(
            &self,
            project_id: &ProjectId,
            user_id: &UserId,
        ) -> Result<Option<User>, AppError> {
            let users = self.users.lock().unwrap();
            Ok(users
                .iter()
                .find(|u| u.project_id == *project_id && u.id == *user_id && u.deleted_at.is_none())
                .cloned())
        }

        async fn find_by_email(
            &self,
            project_id: &ProjectId,
            email: &str,
        ) -> Result<Option<User>, AppError> {
            let users = self.users.lock().unwrap();
            Ok(users
                .iter()
                .find(|u| u.project_id == *project_id && u.email == email && u.deleted_at.is_none())
                .cloned())
        }

        async fn find_by_username(
            &self,
            project_id: &ProjectId,
            username: &str,
        ) -> Result<Option<User>, AppError> {
            let users = self.users.lock().unwrap();
            Ok(users
                .iter()
                .find(|u| {
                    u.project_id == *project_id
                        && u.username.as_deref() == Some(username)
                        && u.deleted_at.is_none()
                })
                .cloned())
        }

        async fn update(
            &self,
            project_id: &ProjectId,
            user_id: &UserId,
            update: &UpdateUser,
        ) -> Result<User, AppError> {
            let mut users = self.users.lock().unwrap();
            let user = users
                .iter_mut()
                .find(|u| u.project_id == *project_id && u.id == *user_id && u.deleted_at.is_none())
                .ok_or_else(|| AppError::Internal(anyhow::anyhow!("user not found")))?;
            if let Some(ref email) = update.email {
                user.email = email.clone();
            }
            if let Some(ref first_name) = update.first_name {
                user.first_name = Some(first_name.clone());
            }
            if let Some(ref last_name) = update.last_name {
                user.last_name = Some(last_name.clone());
            }
            user.updated_at = Utc::now();
            Ok(user.clone())
        }

        async fn soft_delete(
            &self,
            project_id: &ProjectId,
            user_id: &UserId,
        ) -> Result<(), AppError> {
            let mut users = self.users.lock().unwrap();
            if let Some(user) = users
                .iter_mut()
                .find(|u| u.project_id == *project_id && u.id == *user_id)
            {
                user.deleted_at = Some(Utc::now());
            }
            Ok(())
        }

        async fn list(
            &self,
            project_id: &ProjectId,
            params: &PaginationParams,
        ) -> Result<PaginatedResponse<User>, AppError> {
            let users = self.users.lock().unwrap();
            let filtered: Vec<User> = users
                .iter()
                .filter(|u| u.project_id == *project_id && u.deleted_at.is_none())
                .cloned()
                .collect();
            let limit = params.effective_limit();
            let data: Vec<User> = filtered.into_iter().take(limit as usize).collect();
            Ok(PaginatedResponse {
                data,
                has_more: false,
                next_cursor: None,
                total_count: None,
            })
        }

        async fn ban(&self, project_id: &ProjectId, user_id: &UserId) -> Result<(), AppError> {
            let mut users = self.users.lock().unwrap();
            if let Some(user) = users
                .iter_mut()
                .find(|u| u.project_id == *project_id && u.id == *user_id)
            {
                user.banned_at = Some(Utc::now());
            }
            Ok(())
        }

        async fn unban(&self, project_id: &ProjectId, user_id: &UserId) -> Result<(), AppError> {
            let mut users = self.users.lock().unwrap();
            if let Some(user) = users
                .iter_mut()
                .find(|u| u.project_id == *project_id && u.id == *user_id)
            {
                user.banned_at = None;
            }
            Ok(())
        }
    }

    fn make_new_user(email: &str) -> NewUser {
        NewUser {
            email: email.to_string(),
            username: None,
            first_name: Some("Test".to_string()),
            last_name: Some("User".to_string()),
            external_id: None,
            phone: None,
            avatar_url: None,
            metadata: None,
        }
    }

    #[tokio::test]
    async fn create_user_returns_user_with_correct_fields() {
        let repo = MockUserRepo::new();
        let project_id = ProjectId::new();
        let user = repo
            .create(&project_id, &make_new_user("test@example.com"))
            .await
            .unwrap();

        assert_eq!(user.email, "test@example.com");
        assert_eq!(user.first_name, Some("Test".to_string()));
        assert_eq!(user.project_id, project_id);
        assert!(!user.id.0.is_nil());
    }

    #[tokio::test]
    async fn find_by_email_returns_created_user() {
        let repo = MockUserRepo::new();
        let project_id = ProjectId::new();
        repo.create(&project_id, &make_new_user("found@example.com"))
            .await
            .unwrap();

        let found = repo
            .find_by_email(&project_id, "found@example.com")
            .await
            .unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().email, "found@example.com");
    }

    #[tokio::test]
    async fn find_by_email_returns_none_for_nonexistent() {
        let repo = MockUserRepo::new();
        let project_id = ProjectId::new();
        let result = repo
            .find_by_email(&project_id, "nobody@example.com")
            .await
            .unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn find_by_email_respects_project_isolation() {
        let repo = MockUserRepo::new();
        let project_a = ProjectId::new();
        let project_b = ProjectId::new();
        repo.create(&project_a, &make_new_user("shared@example.com"))
            .await
            .unwrap();

        let found_in_a = repo
            .find_by_email(&project_a, "shared@example.com")
            .await
            .unwrap();
        assert!(found_in_a.is_some());

        let found_in_b = repo
            .find_by_email(&project_b, "shared@example.com")
            .await
            .unwrap();
        assert!(found_in_b.is_none());
    }

    #[tokio::test]
    async fn soft_delete_excludes_from_find() {
        let repo = MockUserRepo::new();
        let project_id = ProjectId::new();
        let user = repo
            .create(&project_id, &make_new_user("delete@example.com"))
            .await
            .unwrap();

        repo.soft_delete(&project_id, &user.id).await.unwrap();

        let found_by_id = repo.find_by_id(&project_id, &user.id).await.unwrap();
        assert!(found_by_id.is_none());

        let found_by_email = repo
            .find_by_email(&project_id, "delete@example.com")
            .await
            .unwrap();
        assert!(found_by_email.is_none());
    }

    #[tokio::test]
    async fn ban_and_unban_updates_banned_at() {
        let repo = MockUserRepo::new();
        let project_id = ProjectId::new();
        let user = repo
            .create(&project_id, &make_new_user("ban@example.com"))
            .await
            .unwrap();

        repo.ban(&project_id, &user.id).await.unwrap();
        let banned = repo
            .find_by_id(&project_id, &user.id)
            .await
            .unwrap()
            .unwrap();
        assert!(banned.banned_at.is_some());

        repo.unban(&project_id, &user.id).await.unwrap();
        let unbanned = repo
            .find_by_id(&project_id, &user.id)
            .await
            .unwrap()
            .unwrap();
        assert!(unbanned.banned_at.is_none());
    }

    #[tokio::test]
    async fn update_modifies_user_fields() {
        let repo = MockUserRepo::new();
        let project_id = ProjectId::new();
        let user = repo
            .create(&project_id, &make_new_user("update@example.com"))
            .await
            .unwrap();

        let update = UpdateUser {
            email: Some("new@example.com".to_string()),
            username: None,
            first_name: Some("Updated".to_string()),
            last_name: None,
            avatar_url: None,
            metadata: None,
            private_metadata: None,
        };

        let updated = repo.update(&project_id, &user.id, &update).await.unwrap();
        assert_eq!(updated.email, "new@example.com");
        assert_eq!(updated.first_name, Some("Updated".to_string()));
    }

    #[tokio::test]
    async fn list_excludes_soft_deleted_users() {
        let repo = MockUserRepo::new();
        let project_id = ProjectId::new();
        let user1 = repo
            .create(&project_id, &make_new_user("a@example.com"))
            .await
            .unwrap();
        repo.create(&project_id, &make_new_user("b@example.com"))
            .await
            .unwrap();

        repo.soft_delete(&project_id, &user1.id).await.unwrap();

        let params = PaginationParams {
            limit: 20,
            cursor: None,
        };
        let result = repo.list(&project_id, &params).await.unwrap();
        assert_eq!(result.data.len(), 1);
        assert_eq!(result.data[0].email, "b@example.com");
    }
}
