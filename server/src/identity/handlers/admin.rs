use crate::core::error::AppError;
use crate::core::pagination::{PaginatedResponse, PaginationParams};
use crate::core::types::{ProjectId, UserId};
use crate::db::repos::user_repo::{NewUser, UpdateUser, User};
use serde::Deserialize;

use crate::identity::user::UserService;

/// Request body for creating a user via admin API.
#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub email: String,
    pub username: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub external_id: Option<String>,
    pub phone: Option<String>,
    pub avatar_url: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

/// Request body for updating a user via admin API.
#[derive(Debug, Deserialize)]
pub struct AdminUpdateUserRequest {
    pub email: Option<String>,
    pub username: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub avatar_url: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub private_metadata: Option<serde_json::Value>,
}

/// List users (paginated).
pub async fn list_users(
    user_service: &UserService,
    project_id: &ProjectId,
    params: &PaginationParams,
) -> Result<PaginatedResponse<User>, AppError> {
    user_service.list_users(project_id, params).await
}

/// Create a new user.
pub async fn create_user(
    user_service: &UserService,
    project_id: &ProjectId,
    req: CreateUserRequest,
) -> Result<User, AppError> {
    let new_user = NewUser {
        email: req.email,
        username: req.username,
        first_name: req.first_name,
        last_name: req.last_name,
        external_id: req.external_id,
        phone: req.phone,
        avatar_url: req.avatar_url,
        metadata: req.metadata,
    };
    user_service.create_user(project_id, &new_user).await
}

/// Get a user by ID.
pub async fn get_user(
    user_service: &UserService,
    project_id: &ProjectId,
    user_id: &UserId,
) -> Result<User, AppError> {
    user_service.get_user(project_id, user_id).await
}

/// Update a user by ID.
pub async fn update_user(
    user_service: &UserService,
    project_id: &ProjectId,
    user_id: &UserId,
    req: AdminUpdateUserRequest,
) -> Result<User, AppError> {
    let update = UpdateUser {
        email: req.email,
        username: req.username,
        first_name: req.first_name,
        last_name: req.last_name,
        avatar_url: req.avatar_url,
        metadata: req.metadata,
        private_metadata: req.private_metadata,
    };
    user_service.update_me(project_id, user_id, &update).await
}

/// Soft-delete a user by ID.
pub async fn delete_user(
    user_service: &UserService,
    project_id: &ProjectId,
    user_id: &UserId,
) -> Result<(), AppError> {
    user_service.delete_me(project_id, user_id).await
}

/// Ban a user.
pub async fn ban_user(
    user_service: &UserService,
    project_id: &ProjectId,
    user_id: &UserId,
) -> Result<(), AppError> {
    user_service.ban_user(project_id, user_id).await
}

/// Unban a user.
pub async fn unban_user(
    user_service: &UserService,
    project_id: &ProjectId,
    user_id: &UserId,
) -> Result<(), AppError> {
    user_service.unban_user(project_id, user_id).await
}
