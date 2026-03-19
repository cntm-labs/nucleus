use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct ConfigureProviderRequest {
    pub provider: String,
    pub client_id: String,
    pub client_secret: String,
    pub scopes: Option<Vec<String>>,
    pub is_enabled: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct ProviderResponse {
    pub id: String,
    pub provider: String,
    pub client_id: String, // secret is never returned
    pub scopes: Vec<String>,
    pub is_enabled: bool,
}
