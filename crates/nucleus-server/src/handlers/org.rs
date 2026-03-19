use axum::Json;
use nucleus_core::error::AppError;

// ---------------------------------------------------------------------------
// Phase 4: Organization routes (thin wrappers / direct delegation)
// ---------------------------------------------------------------------------

/// GET /api/v1/orgs
pub async fn handle_list_orgs() -> Result<Json<serde_json::Value>, AppError> {
    nucleus_org::handlers::org::handle_list_orgs().await
}

/// POST /api/v1/orgs
pub async fn handle_create_org(
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    nucleus_org::handlers::org::handle_create_org(Json(req)).await
}

/// GET /api/v1/orgs/:slug
pub async fn handle_get_org() -> Result<Json<serde_json::Value>, AppError> {
    nucleus_org::handlers::org::handle_get_org().await
}

/// PATCH /api/v1/orgs/:slug
pub async fn handle_update_org(
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    nucleus_org::handlers::org::handle_update_org(Json(req)).await
}

// ---------------------------------------------------------------------------
// Members
// ---------------------------------------------------------------------------

/// GET /api/v1/orgs/:slug/members
pub async fn handle_list_members() -> Result<Json<serde_json::Value>, AppError> {
    nucleus_org::handlers::member::handle_list_members().await
}

/// DELETE /api/v1/orgs/:slug/members/:user_id
pub async fn handle_remove_member() -> Result<axum::http::StatusCode, AppError> {
    nucleus_org::handlers::member::handle_remove_member().await
}

/// PATCH /api/v1/orgs/:slug/members/:user_id/role
pub async fn handle_change_role(
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    nucleus_org::handlers::member::handle_change_role(Json(req)).await
}

// ---------------------------------------------------------------------------
// Invitations
// ---------------------------------------------------------------------------

/// GET /api/v1/orgs/:slug/invitations
pub async fn handle_list_invitations() -> Result<Json<serde_json::Value>, AppError> {
    nucleus_org::handlers::invitation::handle_list_invitations().await
}

/// POST /api/v1/orgs/:slug/invitations
pub async fn handle_create_invitation(
    Json(req): Json<nucleus_org::handlers::invitation::CreateInvitationRequest>,
) -> Result<Json<nucleus_org::handlers::invitation::InvitationResponse>, AppError> {
    nucleus_org::handlers::invitation::handle_create_invitation(Json(req)).await
}

/// POST /api/v1/orgs/:slug/invitations/:id/accept
pub async fn handle_accept_invitation() -> Result<Json<serde_json::Value>, AppError> {
    nucleus_org::handlers::invitation::handle_accept_invitation().await
}

/// DELETE /api/v1/orgs/:slug/invitations/:id
pub async fn handle_revoke_invitation() -> Result<axum::http::StatusCode, AppError> {
    nucleus_org::handlers::invitation::handle_revoke_invitation().await
}
