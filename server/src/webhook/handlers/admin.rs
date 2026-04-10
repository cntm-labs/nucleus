use crate::core::error::AppError;
use axum::Json;
use serde_json::json;

/// GET /api/v1/admin/webhooks/events -- list webhook events
pub async fn handle_list_webhook_events() -> Result<Json<serde_json::Value>, AppError> {
    Ok(Json(json!({
        "data": [],
        "has_more": false
    })))
}

/// POST /api/v1/admin/webhooks/events/:id/retry -- retry delivery
pub async fn handle_retry_webhook() -> Result<Json<serde_json::Value>, AppError> {
    Ok(Json(json!({
        "message": "Webhook retry queued"
    })))
}

/// GET /api/v1/admin/webhooks/events/:id/logs -- delivery logs
pub async fn handle_webhook_delivery_logs() -> Result<Json<serde_json::Value>, AppError> {
    Ok(Json(json!({
        "data": []
    })))
}
