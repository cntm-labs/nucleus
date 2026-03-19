use std::sync::Arc;

use axum::{middleware, routing::get, Json, Router};
use serde_json::json;
use tower_http::trace::TraceLayer;

use crate::middleware::request_id::request_id_middleware;
use crate::state::AppState;

pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .layer(middleware::from_fn(request_id_middleware))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

async fn health_check() -> Json<serde_json::Value> {
    Json(json!({
        "status": "ok",
        "service": "nucleus",
        "version": env!("CARGO_PKG_VERSION")
    }))
}
