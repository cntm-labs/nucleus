use crate::core::error::AppError;
use axum::Json;

// ---------------------------------------------------------------------------
// Phase 5: Webhook admin routes (thin wrappers)
// ---------------------------------------------------------------------------

/// GET /api/v1/admin/webhooks/events
pub async fn handle_list_webhook_events() -> Result<Json<serde_json::Value>, AppError> {
    crate::webhook::handlers::admin::handle_list_webhook_events().await
}

/// POST /api/v1/admin/webhooks/events/:id/retry
pub async fn handle_retry_webhook() -> Result<Json<serde_json::Value>, AppError> {
    crate::webhook::handlers::admin::handle_retry_webhook().await
}

/// GET /api/v1/admin/webhooks/events/:id/logs
pub async fn handle_webhook_delivery_logs() -> Result<Json<serde_json::Value>, AppError> {
    crate::webhook::handlers::admin::handle_webhook_delivery_logs().await
}
