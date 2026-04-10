use std::sync::Arc;

use crate::core::types::ProjectId;
use axum::{
    extract::{Request, State},
    middleware::{self, Next},
    response::Response,
    routing::{delete, get, patch, post},
    Json, Router,
};
use serde_json::json;
use tower_http::cors::{AllowHeaders, AllowMethods, AllowOrigin, CorsLayer};
use tower_http::trace::TraceLayer;
use uuid::Uuid;

use crate::handlers::admin;
use crate::handlers::auth;
use crate::handlers::dashboard;
use crate::handlers::identity;
use crate::handlers::org;
use crate::handlers::webhook;
use crate::handlers::well_known;
use crate::middleware::auth::PublicKeyPem;
use crate::middleware::metrics::{handle_metrics, METRICS_ENDPOINT};
use crate::middleware::rate_limit::{make_rate_limit_layer, RateLimitConfig};
use crate::middleware::request_id::request_id_middleware;
use crate::state::AppState;

pub fn create_router(
    state: Arc<AppState>,
    auth_rate_limit: RateLimitConfig,
    api_rate_limit: RateLimitConfig,
    trusted_proxies: Vec<std::net::IpAddr>,
) -> Router {
    let trusted_proxies = Arc::new(trusted_proxies);
    let auth_rate_limiter = make_rate_limit_layer(
        Arc::new(state.redis.clone()),
        auth_rate_limit,
        trusted_proxies.clone(),
        "auth".to_string(),
    );
    let api_rate_limiter = make_rate_limit_layer(
        Arc::new(state.redis.clone()),
        api_rate_limit,
        trusted_proxies,
        "api".to_string(),
    );

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
        .route(
            "/passkey/register/begin",
            post(auth::handle_passkey_register_begin),
        )
        .route(
            "/passkey/register/finish",
            post(auth::handle_passkey_register_finish),
        )
        .route(
            "/passkey/authenticate/begin",
            post(auth::handle_passkey_auth_begin),
        )
        .route(
            "/passkey/authenticate/finish",
            post(auth::handle_passkey_auth_finish),
        )
        // Phase 3: Password Reset
        .route("/password/reset", post(auth::handle_request_reset))
        .route("/password/reset/confirm", post(auth::handle_confirm_reset))
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            inject_auth_context,
        ))
        .layer(axum::middleware::from_fn(auth_rate_limiter));

    // Phase 4: User profile routes (authenticated)
    let user_routes = Router::new()
        .route(
            "/me",
            get(identity::handle_get_me)
                .patch(identity::handle_update_me)
                .delete(identity::handle_delete_me),
        )
        .route("/me/sessions", get(identity::handle_list_my_sessions))
        .route(
            "/me/sessions/{id}",
            delete(identity::handle_revoke_my_session),
        )
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            inject_auth_context,
        ));

    // Phase 4: Organization routes (authenticated)
    let org_routes = Router::new()
        .route("/", get(org::handle_list_orgs).post(org::handle_create_org))
        .route(
            "/{slug}",
            get(org::handle_get_org).patch(org::handle_update_org),
        )
        .route("/{slug}/members", get(org::handle_list_members))
        .route(
            "/{slug}/members/{user_id}",
            delete(org::handle_remove_member),
        )
        .route(
            "/{slug}/members/{user_id}/role",
            patch(org::handle_change_role),
        )
        .route(
            "/{slug}/invitations",
            get(org::handle_list_invitations).post(org::handle_create_invitation),
        )
        .route(
            "/{slug}/invitations/{id}/accept",
            post(org::handle_accept_invitation),
        )
        .route(
            "/{slug}/invitations/{id}",
            delete(org::handle_revoke_invitation),
        )
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            inject_auth_context,
        ))
        .layer(axum::middleware::from_fn(api_rate_limiter));

    // Phase 4: Admin routes (require secret API key)
    let admin_routes = Router::new()
        .route(
            "/users",
            get(admin::handle_admin_list_users).post(admin::handle_admin_create_user),
        )
        .route(
            "/users/{id}",
            get(admin::handle_admin_get_user)
                .patch(admin::handle_admin_update_user)
                .delete(admin::handle_admin_delete_user),
        )
        .route("/users/{id}/ban", post(admin::handle_admin_ban_user))
        .route("/users/{id}/unban", post(admin::handle_admin_unban_user));

    // Phase 5: Webhook admin routes (merged into admin)
    let webhook_admin_routes = Router::new()
        .route("/webhooks/events", get(webhook::handle_list_webhook_events))
        .route(
            "/webhooks/events/{id}/retry",
            post(webhook::handle_retry_webhook),
        )
        .route(
            "/webhooks/events/{id}/logs",
            get(webhook::handle_webhook_delivery_logs),
        );

    // Phase 5: Dashboard API routes (require account session)
    let dashboard_routes = Router::new()
        // Projects
        .route(
            "/projects",
            get(dashboard::handle_list_projects).post(dashboard::handle_create_project),
        )
        .route(
            "/projects/{id}",
            get(dashboard::handle_get_project).patch(dashboard::handle_update_project),
        )
        // OAuth providers
        .route(
            "/projects/{id}/providers",
            get(dashboard::handle_list_providers).post(dashboard::handle_configure_provider),
        )
        .route(
            "/projects/{id}/providers/{provider_id}",
            delete(dashboard::handle_delete_provider),
        )
        // API keys
        .route(
            "/projects/{id}/api-keys",
            get(dashboard::handle_list_api_keys).post(dashboard::handle_create_api_key),
        )
        .route(
            "/projects/{id}/api-keys/{key_id}",
            delete(dashboard::handle_revoke_api_key),
        )
        // Signing keys
        .route(
            "/projects/{id}/signing-keys",
            get(dashboard::handle_list_signing_keys),
        )
        .route(
            "/projects/{id}/signing-keys/rotate",
            post(dashboard::handle_rotate_signing_key),
        )
        // Templates
        .route(
            "/projects/{id}/templates",
            get(dashboard::handle_list_templates),
        )
        .route(
            "/projects/{id}/templates/{template_id}",
            patch(dashboard::handle_update_template),
        )
        .route(
            "/projects/{id}/templates/{template_id}/reset",
            post(dashboard::handle_reset_template),
        )
        // JWT templates
        .route(
            "/projects/{id}/jwt-templates",
            get(dashboard::handle_list_jwt_templates).post(dashboard::handle_create_jwt_template),
        )
        .route(
            "/projects/{id}/jwt-templates/{jt_id}",
            patch(dashboard::handle_update_jwt_template),
        )
        // Analytics
        .route(
            "/projects/{id}/analytics",
            get(dashboard::handle_get_analytics),
        )
        // Billing
        .route("/projects/{id}/usage", get(dashboard::handle_get_usage))
        .route(
            "/projects/{id}/subscription",
            get(dashboard::handle_get_subscription),
        )
        // Audit logs
        .route(
            "/projects/{id}/audit-logs",
            get(dashboard::handle_list_audit_logs),
        )
        // Settings
        .route(
            "/projects/{id}/settings",
            get(dashboard::handle_get_settings).patch(dashboard::handle_update_settings),
        );

    Router::new()
        .route("/.well-known/jwks.json", get(well_known::handle_jwks))
        .route(
            "/.well-known/openid-configuration",
            get(well_known::handle_openid_configuration),
        )
        .route("/health", get(health_check))
        .route(METRICS_ENDPOINT, get(handle_metrics))
        .nest("/api/v1/auth", auth_routes)
        .nest("/api/v1/users", user_routes)
        .nest("/api/v1/orgs", org_routes)
        .nest("/api/v1/admin", admin_routes.merge(webhook_admin_routes))
        .nest("/api/v1/dashboard", dashboard_routes)
        .layer(build_cors_layer(&state.allowed_origins))
        .layer(middleware::from_fn(request_id_middleware))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

/// Middleware that injects auth context (public key, project ID, session service)
/// into request extensions so that JwtAuth extractor can validate tokens.
async fn inject_auth_context(
    State(state): State<Arc<AppState>>,
    mut req: Request,
    next: Next,
) -> Response {
    req.extensions_mut()
        .insert(PublicKeyPem(state.signing_key.public_key_pem.clone()));
    req.extensions_mut()
        .insert(ProjectId::from_uuid(Uuid::nil())); // System project
    req.extensions_mut().insert(state.session_service.clone());
    next.run(req).await
}

fn build_cors_layer(allowed_origins: &[String]) -> CorsLayer {
    let origins = if allowed_origins.is_empty() {
        // Default: restrictive — no cross-origin requests allowed
        AllowOrigin::list(std::iter::empty::<axum::http::HeaderValue>())
    } else {
        AllowOrigin::list(
            allowed_origins
                .iter()
                .filter_map(|o| o.parse::<axum::http::HeaderValue>().ok()),
        )
    };

    CorsLayer::new()
        .allow_origin(origins)
        .allow_methods(AllowMethods::list([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::PATCH,
            axum::http::Method::DELETE,
            axum::http::Method::OPTIONS,
        ]))
        .allow_headers(AllowHeaders::list([
            axum::http::header::AUTHORIZATION,
            axum::http::header::CONTENT_TYPE,
            axum::http::header::ACCEPT,
        ]))
        .allow_credentials(true)
        .max_age(std::time::Duration::from_secs(3600))
}

/// Server start time for uptime calculation
static SERVER_START: std::sync::OnceLock<chrono::DateTime<chrono::Utc>> =
    std::sync::OnceLock::new();

async fn health_check(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let start = SERVER_START.get_or_init(chrono::Utc::now);

    let db_ok = sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(&state.db)
        .await
        .is_ok();

    let redis_ok = {
        let mut conn = state.redis.clone();
        redis::cmd("PING")
            .query_async::<String>(&mut conn)
            .await
            .is_ok()
    };

    let now = state.clock.now();
    let uptime_secs = (now - *start).num_seconds();

    Json(json!({
        "status": if db_ok && redis_ok { "ok" } else { "degraded" },
        "service": "nucleus",
        "version": env!("CARGO_PKG_VERSION"),
        "uptime_seconds": uptime_secs,
        "checks": {
            "database": db_ok,
            "redis": redis_ok,
        },
        "server_time": now.to_rfc3339(),
        "master_key_configured": !state.master_key.iter().all(|&b| b == 0),
    }))
}
