// Placeholder for Prometheus metrics middleware
// In production, use metrics + metrics-exporter-prometheus crates

use axum::Json;
use serde_json::json;

/// The path where the metrics endpoint is mounted.
pub const METRICS_ENDPOINT: &str = "/metrics";

/// Placeholder metrics handler that returns basic service info.
/// In production, replace with metrics-exporter-prometheus output.
pub async fn handle_metrics() -> Json<serde_json::Value> {
    Json(json!({
        "status": "ok",
        "message": "Metrics endpoint placeholder — integrate prometheus exporter"
    }))
}
