use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    #[serde(default = "default_limit")]
    pub limit: u32,
    pub cursor: Option<String>,
}

fn default_limit() -> u32 {
    20
}

impl PaginationParams {
    pub fn effective_limit(&self) -> u32 {
        self.limit.min(100).max(1)
    }
}

#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T: Serialize> {
    pub data: Vec<T>,
    pub has_more: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_count: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_limit_is_20() {
        let params: PaginationParams = serde_json::from_str("{}").unwrap();
        assert_eq!(params.limit, 20);
    }

    #[test]
    fn effective_limit_capped_at_100() {
        let params: PaginationParams = serde_json::from_str(r#"{"limit": 500}"#).unwrap();
        assert_eq!(params.effective_limit(), 100);
    }

    #[test]
    fn effective_limit_minimum_1() {
        let params: PaginationParams = serde_json::from_str(r#"{"limit": 0}"#).unwrap();
        assert_eq!(params.effective_limit(), 1);
    }

    #[test]
    fn paginated_response_serializes_correctly() {
        let response = PaginatedResponse {
            data: vec!["item1", "item2"],
            has_more: true,
            next_cursor: Some("cursor_abc".to_string()),
            total_count: Some(42),
        };
        let json = serde_json::to_value(&response).unwrap();
        assert_eq!(json["data"], serde_json::json!(["item1", "item2"]));
        assert_eq!(json["has_more"], true);
        assert_eq!(json["next_cursor"], "cursor_abc");
        assert_eq!(json["total_count"], 42);
    }

    #[test]
    fn paginated_response_skips_none_fields() {
        let response = PaginatedResponse {
            data: vec![1, 2, 3],
            has_more: false,
            next_cursor: None,
            total_count: None,
        };
        let json = serde_json::to_value(&response).unwrap();
        assert!(json.get("next_cursor").is_none());
        assert!(json.get("total_count").is_none());
        assert_eq!(json["has_more"], false);
    }
}
