use axum::Json;
use nucleus_core::error::AppError;

/// GET /api/v1/admin/webhooks/events -- list webhook events
pub async fn handle_list_webhook_events() -> Result<Json<serde_json::Value>, AppError> {
    todo!()
}

/// POST /api/v1/admin/webhooks/events/:id/retry -- retry delivery
pub async fn handle_retry_webhook() -> Result<Json<serde_json::Value>, AppError> {
    todo!()
}

/// GET /api/v1/admin/webhooks/events/:id/logs -- delivery logs
pub async fn handle_webhook_delivery_logs() -> Result<Json<serde_json::Value>, AppError> {
    todo!()
}
