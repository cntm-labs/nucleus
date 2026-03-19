use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct SigningKeyResponse {
    pub id: String,
    pub algorithm: String,
    pub is_current: bool,
    pub created_at: String,
    pub rotated_at: Option<String>,
}
