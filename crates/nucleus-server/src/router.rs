use std::sync::Arc;

use axum::{middleware, routing::{get, post}, Json, Router};
use serde_json::json;
use tower_http::trace::TraceLayer;

use crate::handlers::auth;
use crate::middleware::request_id::request_id_middleware;
use crate::state::AppState;

pub fn create_router(state: Arc<AppState>) -> Router {
    let auth_routes = Router::new()
        // Phase 2: Core auth
        .route("/sign-up", post(auth::handle_sign_up))
        .route("/sign-in", post(auth::handle_sign_in))
        .route("/token/refresh", post(auth::handle_refresh))
        .route("/sign-out", post(auth::handle_sign_out))
        .route("/sign-out/all", post(auth::handle_sign_out_all))
        // Phase 3: OAuth
        .route("/sign-in/oauth", post(auth::handle_oauth_start))
        .route("/oauth/callback", get(auth::handle_oauth_callback))
        // Phase 3: Magic Link
        .route("/sign-in/magic-link", post(auth::handle_send_magic_link))
        .route("/magic-link/verify", get(auth::handle_verify_magic_link))
        // Phase 3: OTP
        .route("/sign-in/otp/send", post(auth::handle_send_otp))
        .route("/sign-in/otp/verify", post(auth::handle_verify_otp))
        // Phase 3: MFA
        .route("/mfa/verify", post(auth::handle_mfa_verify))
        // Phase 3: Passkeys
        .route("/passkey/register/begin", post(auth::handle_passkey_register_begin))
        .route("/passkey/register/finish", post(auth::handle_passkey_register_finish))
        .route("/passkey/authenticate/begin", post(auth::handle_passkey_auth_begin))
        .route("/passkey/authenticate/finish", post(auth::handle_passkey_auth_finish))
        // Phase 3: Password Reset
        .route("/password/reset", post(auth::handle_request_reset))
        .route("/password/reset/confirm", post(auth::handle_confirm_reset));

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
