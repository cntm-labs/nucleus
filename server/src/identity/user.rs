use std::sync::Arc;

use crate::core::error::{AppError, UserError};
use crate::core::pagination::{PaginatedResponse, PaginationParams};
use crate::core::types::{ProjectId, UserId};
use crate::db::repos::user_repo::{UpdateUser, User, UserRepository};

pub struct UserService {
    user_repo: Arc<dyn UserRepository>,
}

impl UserService {
    pub fn new(user_repo: Arc<dyn UserRepository>) -> Self {
        Self { user_repo }
    }

    /// Get current user profile
    pub async fn get_me(&self, project_id: &ProjectId, user_id: &UserId) -> Result<User, AppError> {
        self.user_repo
            .find_by_id(project_id, user_id)
            .await?
            .ok_or(AppError::User(UserError::NotFound))
    }

    /// Update current user profile
    pub async fn update_me(
        &self,
        project_id: &ProjectId,
        user_id: &UserId,
        update: &UpdateUser,
    ) -> Result<User, AppError> {
        // Verify user exists
        self.get_me(project_id, user_id).await?;
        self.user_repo.update(project_id, user_id, update).await
    }

    /// Soft delete own account
    pub async fn delete_me(
        &self,
        project_id: &ProjectId,
        user_id: &UserId,
    ) -> Result<(), AppError> {
        self.user_repo.soft_delete(project_id, user_id).await
    }

    // ── Admin methods ──

    /// Create a new user (admin)
    pub async fn create_user(
        &self,
        project_id: &ProjectId,
        new_user: &crate::db::repos::user_repo::NewUser,
    ) -> Result<User, AppError> {
        self.user_repo.create(project_id, new_user).await
    }

    /// List users (admin)
    pub async fn list_users(
        &self,
        project_id: &ProjectId,
        params: &PaginationParams,
    ) -> Result<PaginatedResponse<User>, AppError> {
        self.user_repo.list(project_id, params).await
    }

    /// Get user by ID (admin)
    pub async fn get_user(
        &self,
        project_id: &ProjectId,
        user_id: &UserId,
    ) -> Result<User, AppError> {
        self.get_me(project_id, user_id).await
    }

    /// Ban user (admin)
    pub async fn ban_user(&self, project_id: &ProjectId, user_id: &UserId) -> Result<(), AppError> {
        self.user_repo.ban(project_id, user_id).await
    }

    /// Unban user (admin)
    pub async fn unban_user(
        &self,
        project_id: &ProjectId,
        user_id: &UserId,
    ) -> Result<(), AppError> {
        self.user_repo.unban(project_id, user_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::repos::user_repo::NewUser;
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

        fn with_users(users: Vec<User>) -> Self {
            Self {
                users: Mutex::new(users),
            }
        }
    }

    fn make_user(project_id: ProjectId, user_id: UserId, email: &str) -> User {
        let now = Utc::now();
        User {
            id: user_id,
            project_id,
            external_id: None,
            email: email.to_string(),
            email_verified: false,
            phone: None,
            phone_verified: false,
            username: None,
            first_name: None,
            last_name: None,
            avatar_url: None,
            metadata: serde_json::json!({}),
            private_metadata: serde_json::json!({}),
            last_sign_in_at: None,
            banned_at: None,
            deleted_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    #[async_trait]
    impl UserRepository for MockUserRepo {
        async fn create(
            &self,
            project_id: &ProjectId,
            new_user: &NewUser,
        ) -> Result<User, AppError> {
            let user = make_user(*project_id, UserId::new(), &new_user.email);
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
                .find(|u| u.id == *user_id && u.project_id == *project_id && u.deleted_at.is_none())
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
                .find(|u| u.email == email && u.project_id == *project_id && u.deleted_at.is_none())
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
                    u.username.as_deref() == Some(username)
                        && u.project_id == *project_id
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
                .find(|u| u.id == *user_id && u.project_id == *project_id && u.deleted_at.is_none())
                .ok_or(AppError::User(UserError::NotFound))?;

            if let Some(ref email) = update.email {
                user.email = email.clone();
            }
            if let Some(ref username) = update.username {
                user.username = Some(username.clone());
            }
            if let Some(ref first_name) = update.first_name {
                user.first_name = Some(first_name.clone());
            }
            if let Some(ref last_name) = update.last_name {
                user.last_name = Some(last_name.clone());
            }
            if let Some(ref avatar_url) = update.avatar_url {
                user.avatar_url = Some(avatar_url.clone());
            }
            if let Some(ref metadata) = update.metadata {
                user.metadata = metadata.clone();
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
                .find(|u| u.id == *user_id && u.project_id == *project_id && u.deleted_at.is_none())
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
            let limit = params.effective_limit() as usize;
            let data: Vec<User> = filtered.into_iter().take(limit).collect();
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
                .find(|u| u.id == *user_id && u.project_id == *project_id && u.deleted_at.is_none())
            {
                user.banned_at = Some(Utc::now());
            }
            Ok(())
        }

        async fn unban(&self, project_id: &ProjectId, user_id: &UserId) -> Result<(), AppError> {
            let mut users = self.users.lock().unwrap();
            if let Some(user) = users
                .iter_mut()
                .find(|u| u.id == *user_id && u.project_id == *project_id)
            {
                user.banned_at = None;
            }
            Ok(())
        }
    }

    #[tokio::test]
    async fn get_me_returns_user() {
        let project_id = ProjectId::new();
        let user_id = UserId::new();
        let user = make_user(project_id, user_id, "alice@example.com");
        let repo = Arc::new(MockUserRepo::with_users(vec![user.clone()]));
        let svc = UserService::new(repo);

        let result = svc.get_me(&project_id, &user_id).await.unwrap();
        assert_eq!(result.id, user_id);
        assert_eq!(result.email, "alice@example.com");
    }

    #[tokio::test]
    async fn get_me_returns_not_found_for_unknown_user() {
        let project_id = ProjectId::new();
        let user_id = UserId::new();
        let repo = Arc::new(MockUserRepo::new());
        let svc = UserService::new(repo);

        let result = svc.get_me(&project_id, &user_id).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code(), "user/not_found");
    }

    #[tokio::test]
    async fn update_me_updates_fields() {
        let project_id = ProjectId::new();
        let user_id = UserId::new();
        let user = make_user(project_id, user_id, "alice@example.com");
        let repo = Arc::new(MockUserRepo::with_users(vec![user]));
        let svc = UserService::new(repo);

        let update = UpdateUser {
            email: None,
            username: None,
            first_name: Some("Alice".to_string()),
            last_name: Some("Smith".to_string()),
            avatar_url: None,
            metadata: None,
            private_metadata: None,
        };

        let result = svc.update_me(&project_id, &user_id, &update).await.unwrap();
        assert_eq!(result.first_name.as_deref(), Some("Alice"));
        assert_eq!(result.last_name.as_deref(), Some("Smith"));
    }

    #[tokio::test]
    async fn delete_me_soft_deletes() {
        let project_id = ProjectId::new();
        let user_id = UserId::new();
        let user = make_user(project_id, user_id, "alice@example.com");
        let repo = Arc::new(MockUserRepo::with_users(vec![user]));
        let svc = UserService::new(repo);

        svc.delete_me(&project_id, &user_id).await.unwrap();

        // User should no longer be findable
        let result = svc.get_me(&project_id, &user_id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn ban_user_sets_banned_at() {
        let project_id = ProjectId::new();
        let user_id = UserId::new();
        let user = make_user(project_id, user_id, "alice@example.com");
        let repo = Arc::new(MockUserRepo::with_users(vec![user]));
        let svc = UserService::new(repo);

        svc.ban_user(&project_id, &user_id).await.unwrap();

        let result = svc.get_me(&project_id, &user_id).await.unwrap();
        assert!(result.banned_at.is_some());
    }

    #[tokio::test]
    async fn unban_user_clears_banned_at() {
        let project_id = ProjectId::new();
        let user_id = UserId::new();
        let mut user = make_user(project_id, user_id, "alice@example.com");
        user.banned_at = Some(Utc::now());
        let repo = Arc::new(MockUserRepo::with_users(vec![user]));
        let svc = UserService::new(repo);

        svc.unban_user(&project_id, &user_id).await.unwrap();

        let result = svc.get_me(&project_id, &user_id).await.unwrap();
        assert!(result.banned_at.is_none());
    }

    #[tokio::test]
    async fn cross_tenant_isolation_user_not_visible() {
        let project_a = ProjectId::new();
        let project_b = ProjectId::new();
        let user_id = UserId::new();
        let user = make_user(project_a, user_id, "alice@example.com");
        let repo = Arc::new(MockUserRepo::with_users(vec![user]));
        let svc = UserService::new(repo);

        // User in project A should not be found when querying project B
        let result = svc.get_me(&project_b, &user_id).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code(), "user/not_found");
    }
}
