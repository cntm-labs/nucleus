use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct UsageResponse {
    pub current_period: String,
    pub mau_count: u64,
    pub mau_limit: Option<u64>,
    pub api_request_count: u64,
    pub api_request_limit: Option<u64>,
}
