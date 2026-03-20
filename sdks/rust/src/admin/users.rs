use super::{HttpClient, PaginatedResponse};
use crate::verify::NucleusError;
use serde::Deserialize;
use std::sync::Arc;

/// A Nucleus user returned by the Admin API.
#[derive(Debug, Clone, Deserialize)]
pub struct NucleusUser {
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

/// Optional parameters for listing users.
#[derive(Debug, Default)]
pub struct ListUsersParams {
    pub limit: Option<u32>,
    pub cursor: Option<String>,
    pub email_contains: Option<String>,
}

/// Admin Users API.
#[derive(Clone)]
pub struct UsersApi {
    http: Arc<HttpClient>,
}

impl UsersApi {
    pub(crate) fn new(http: Arc<HttpClient>) -> Self {
        Self { http }
    }

    /// Get a single user by ID.
    pub async fn get(&self, user_id: &str) -> Result<NucleusUser, NucleusError> {
        self.http.get(&format!("/users/{user_id}")).await
    }

    /// List users with optional pagination and filtering.
    pub async fn list(
        &self,
        params: Option<ListUsersParams>,
    ) -> Result<PaginatedResponse<NucleusUser>, NucleusError> {
        let mut query_parts: Vec<String> = Vec::new();
        if let Some(p) = &params {
            if let Some(limit) = p.limit {
                query_parts.push(format!("limit={limit}"));
            }
            if let Some(cursor) = &p.cursor {
                query_parts.push(format!("cursor={cursor}"));
            }
            if let Some(email) = &p.email_contains {
                query_parts.push(format!("email_contains={email}"));
            }
        }

        let path = if query_parts.is_empty() {
            "/users".to_string()
        } else {
            format!("/users?{}", query_parts.join("&"))
        };

        self.http.get(&path).await
    }

    /// Delete a user by ID.
    pub async fn delete(&self, user_id: &str) -> Result<(), NucleusError> {
        self.http.delete(&format!("/users/{user_id}")).await
    }

    /// Ban a user by ID.
    pub async fn ban(&self, user_id: &str) -> Result<(), NucleusError> {
        self.http
            .post::<serde_json::Value>(&format!("/users/{user_id}/ban"))
            .await?;
        Ok(())
    }

    /// Unban a user by ID.
    pub async fn unban(&self, user_id: &str) -> Result<(), NucleusError> {
        self.http
            .post::<serde_json::Value>(&format!("/users/{user_id}/unban"))
            .await?;
        Ok(())
    }
}
