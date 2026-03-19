use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct ApiKeyResponse {
    pub id: String,
    pub key_type: String,
    pub key_prefix: String,
    pub label: Option<String>,
    pub scopes: Vec<String>,
    pub last_used_at: Option<String>,
    pub created_at: String,
    pub revoked_at: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ApiKeyCreatedResponse {
    pub id: String,
    pub key: String, // ONLY shown once at creation
    pub key_type: String,
    pub key_prefix: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateApiKeyRequest {
    pub key_type: String, // "publishable" or "secret"
    pub label: Option<String>,
    pub scopes: Option<Vec<String>>,
}
