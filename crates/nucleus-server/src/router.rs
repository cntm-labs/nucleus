use std::sync::Arc;

use axum::{middleware, routing::{delete, get, patch, post}, Json, Router};
use serde_json::json;
use tower_http::trace::TraceLayer;

use crate::handlers::admin;
use crate::handlers::auth;
use crate::handlers::identity;
use crate::handlers::org;
use crate::handlers::well_known;
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

    // Phase 4: User profile routes (authenticated)
    let user_routes = Router::new()
        .route("/me", get(identity::handle_get_me).patch(identity::handle_update_me).delete(identity::handle_delete_me))
        .route("/me/sessions", get(identity::handle_list_my_sessions))
        .route("/me/sessions/:id", delete(identity::handle_revoke_my_session));

    // Phase 4: Organization routes (authenticated)
    let org_routes = Router::new()
        .route("/", get(org::handle_list_orgs).post(org::handle_create_org))
        .route("/:slug", get(org::handle_get_org).patch(org::handle_update_org))
        .route("/:slug/members", get(org::handle_list_members))
        .route("/:slug/members/:user_id", delete(org::handle_remove_member))
        .route("/:slug/members/:user_id/role", patch(org::handle_change_role))
        .route("/:slug/invitations", get(org::handle_list_invitations).post(org::handle_create_invitation))
        .route("/:slug/invitations/:id/accept", post(org::handle_accept_invitation))
        .route("/:slug/invitations/:id", delete(org::handle_revoke_invitation));

    // Phase 4: Admin routes (require secret API key)
    let admin_routes = Router::new()
        .route("/users", get(admin::handle_admin_list_users).post(admin::handle_admin_create_user))
        .route("/users/:id", get(admin::handle_admin_get_user).patch(admin::handle_admin_update_user).delete(admin::handle_admin_delete_user))
        .route("/users/:id/ban", post(admin::handle_admin_ban_user))
        .route("/users/:id/unban", post(admin::handle_admin_unban_user));

    Router::new()
        .route("/.well-known/jwks.json", get(well_known::handle_jwks))
        .route("/.well-known/openid-configuration", get(well_known::handle_openid_configuration))
        .route("/health", get(health_check))
        .nest("/api/v1/auth", auth_routes)
        .nest("/api/v1/users", user_routes)
        .nest("/api/v1/orgs", org_routes)
        .nest("/api/v1/admin", admin_routes)
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
