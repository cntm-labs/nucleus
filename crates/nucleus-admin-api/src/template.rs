use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct UpdateTemplateRequest {
    pub subject: Option<String>,
    pub body_html: Option<String>,
    pub body_text: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct TemplateResponse {
    pub id: String,
    pub template_type: String,
    pub event: String,
    pub subject: Option<String>,
    pub is_custom: bool,
}
