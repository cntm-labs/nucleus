use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct ProjectResponse {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub data_mode: String,
    pub environment: String,
    pub is_active: bool,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateProjectRequest {
    pub name: String,
    pub slug: String,
    pub data_mode: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProjectRequest {
    pub name: Option<String>,
    pub webhook_url: Option<String>,
    pub allowed_origins: Option<Vec<String>>,
    pub session_ttl: Option<i32>,
    pub jwt_lifetime: Option<i32>,
    pub settings: Option<serde_json::Value>,
}
