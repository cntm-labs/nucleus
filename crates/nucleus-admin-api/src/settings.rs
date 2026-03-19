use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct UpdateSettingsRequest {
    pub session_ttl: Option<i32>,
    pub jwt_lifetime: Option<i32>,
    pub jwt_algorithm: Option<String>,
    pub allowed_origins: Option<Vec<String>>,
}
