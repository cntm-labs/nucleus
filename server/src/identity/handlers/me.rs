use crate::core::error::AppError;
use crate::core::types::{ProjectId, UserId};
use crate::db::repos::user_repo::{UpdateUser, User};
use serde::{Deserialize, Serialize};

use crate::identity::user::UserService;

#[derive(Debug, Serialize)]
pub struct UserProfileResponse {
    pub id: String,
    pub email: String,
    pub email_verified: bool,
    pub username: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub avatar_url: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: String,
    pub updated_at: String,
}

impl From<User> for UserProfileResponse {
    fn from(u: User) -> Self {
        Self {
            id: u.id.to_string(),
            email: u.email,
            email_verified: u.email_verified,
            username: u.username,
            first_name: u.first_name,
            last_name: u.last_name,
            avatar_url: u.avatar_url,
            metadata: u.metadata,
            created_at: u.created_at.to_rfc3339(),
            updated_at: u.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdateProfileRequest {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub username: Option<String>,
    pub avatar_url: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

/// Get the current user's profile.
pub async fn get_me(
    user_service: &UserService,
    project_id: &ProjectId,
    user_id: &UserId,
) -> Result<UserProfileResponse, AppError> {
    let user = user_service.get_me(project_id, user_id).await?;
    Ok(UserProfileResponse::from(user))
}

/// Update the current user's profile.
pub async fn update_me(
    user_service: &UserService,
    project_id: &ProjectId,
    user_id: &UserId,
    req: UpdateProfileRequest,
) -> Result<UserProfileResponse, AppError> {
    let update = UpdateUser {
        email: None,
        username: req.username,
        first_name: req.first_name,
        last_name: req.last_name,
        avatar_url: req.avatar_url,
        metadata: req.metadata,
        private_metadata: None,
    };
    let user = user_service.update_me(project_id, user_id, &update).await?;
    Ok(UserProfileResponse::from(user))
}

/// Delete (soft) the current user's account.
pub async fn delete_me(
    user_service: &UserService,
    project_id: &ProjectId,
    user_id: &UserId,
) -> Result<(), AppError> {
    user_service.delete_me(project_id, user_id).await
}
