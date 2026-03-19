use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct AnalyticsResponse {
    pub total_users: u64,
    pub mau: u64,
    pub sign_ins_today: u64,
    pub sign_ins_this_week: u64,
    pub sign_in_methods: serde_json::Value,
}
