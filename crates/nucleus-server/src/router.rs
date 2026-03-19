use std::sync::Arc;

use axum::{middleware, routing::{get, post}, Json, Router};
use serde_json::json;
use tower_http::trace::TraceLayer;

use crate::handlers::auth;
use crate::middleware::request_id::request_id_middleware;
use crate::state::AppState;

pub fn create_router(state: Arc<AppState>) -> Router {
    let auth_routes = Router::new()
        .route("/sign-up", post(auth::handle_sign_up))
        .route("/sign-in", post(auth::handle_sign_in))
        .route("/token/refresh", post(auth::handle_refresh))
        .route("/sign-out", post(auth::handle_sign_out))
        .route("/sign-out/all", post(auth::handle_sign_out_all));

    Router::new()
        .route("/health", get(health_check))
        .nest("/api/v1/auth", auth_routes)
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
