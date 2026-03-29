use chrono::Utc;
use nucleus_core::crypto;
use serde::Serialize;

/// Webhook payload structure
#[derive(Debug, Serialize)]
pub struct WebhookPayload {
    pub id: String,
    pub event_type: String,
    pub timestamp: String,
    pub data: serde_json::Value,
}

/// HMAC-SHA256 signature for webhook delivery
pub struct WebhookSigner;

impl WebhookSigner {
    /// Sign a webhook payload.
    ///
    /// Format: `timestamp.json_body`
    /// Header: `X-Nucleus-Signature: v1=<hex_signature>`
    pub fn sign(webhook_secret: &str, timestamp: i64, body: &str) -> String {
        let payload = format!("{}.{}", timestamp, body);
        let signature = crypto::hmac_sign(webhook_secret.as_bytes(), payload.as_bytes());
        format!("v1={}", signature)
    }

    /// Verify a webhook signature.
    ///
    /// Rejects signatures with timestamps older than 5 minutes.
    pub fn verify(webhook_secret: &str, timestamp: i64, body: &str, signature: &str) -> bool {
        // Check timestamp not too old (5 min tolerance)
        let now = Utc::now().timestamp();
        if (now - timestamp).abs() > 300 {
            return false;
        }

        let expected = Self::sign(webhook_secret, timestamp, body);
        crypto::constant_time_eq(expected.as_bytes(), signature.as_bytes())
    }

    /// Build headers for webhook delivery.
    pub fn build_headers(signature: &str, timestamp: i64, event_id: &str) -> Vec<(String, String)> {
        vec![
            ("Content-Type".to_string(), "application/json".to_string()),
            ("X-Nucleus-Signature".to_string(), signature.to_string()),
            ("X-Nucleus-Timestamp".to_string(), timestamp.to_string()),
            ("X-Nucleus-Event-Id".to_string(), event_id.to_string()),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sign_produces_v1_format() {
        let sig = WebhookSigner::sign("secret", 1000, r#"{"hello":"world"}"#);
        assert!(sig.starts_with("v1="), "signature should start with v1=");
        // v1= + 64 hex chars (SHA-256)
        assert_eq!(sig.len(), 3 + 64);
    }

    #[test]
    fn sign_is_deterministic() {
        let sig1 = WebhookSigner::sign("secret", 1000, "body");
        let sig2 = WebhookSigner::sign("secret", 1000, "body");
        assert_eq!(sig1, sig2);
    }

    #[test]
    fn verify_valid_signature() {
        let now = Utc::now().timestamp();
        let body = r#"{"user_id":"u_123"}"#;
        let sig = WebhookSigner::sign("my-secret", now, body);
        assert!(WebhookSigner::verify("my-secret", now, body, &sig));
    }

    #[test]
    fn verify_rejects_tampered_body() {
        let now = Utc::now().timestamp();
        let sig = WebhookSigner::sign("my-secret", now, "original body");
        assert!(!WebhookSigner::verify(
            "my-secret",
            now,
            "tampered body",
            &sig
        ));
    }

    #[test]
    fn verify_rejects_old_timestamp() {
        let old_timestamp = Utc::now().timestamp() - 600; // 10 minutes ago
        let body = "some body";
        let sig = WebhookSigner::sign("my-secret", old_timestamp, body);
        assert!(!WebhookSigner::verify(
            "my-secret",
            old_timestamp,
            body,
            &sig
        ));
    }

    #[test]
    fn build_headers_includes_all_required() {
        let headers = WebhookSigner::build_headers("v1=abc123", 1700000000, "evt_123");

        let header_map: std::collections::HashMap<&str, &str> = headers
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();

        assert_eq!(header_map.get("Content-Type"), Some(&"application/json"));
        assert_eq!(header_map.get("X-Nucleus-Signature"), Some(&"v1=abc123"));
        assert_eq!(header_map.get("X-Nucleus-Timestamp"), Some(&"1700000000"));
        assert_eq!(header_map.get("X-Nucleus-Event-Id"), Some(&"evt_123"));
        assert_eq!(headers.len(), 4);
    }
}
