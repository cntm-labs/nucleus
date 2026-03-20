use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::Json;
use nucleus_core::error::AppError;

use crate::state::AppState;

// ---------------------------------------------------------------------------
// Phase 4: Organization routes (thin wrappers / direct delegation)
// ---------------------------------------------------------------------------

/// GET /api/v1/orgs
pub async fn handle_list_orgs(
    State(state): State<Arc<AppState>>,
    query: Query<nucleus_org::handlers::org::ListOrgsParams>,
) -> Result<Json<serde_json::Value>, AppError> {
    nucleus_org::handlers::org::handle_list_orgs(State(state.org_service.clone()), query).await
}

/// POST /api/v1/orgs
pub async fn handle_create_org(
    State(state): State<Arc<AppState>>,
    query: Query<nucleus_org::handlers::org::RequestContext>,
    Json(req): Json<nucleus_org::handlers::org::CreateOrgRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    nucleus_org::handlers::org::handle_create_org(
        State(state.org_service.clone()),
        query,
        Json(req),
    )
    .await
}

/// GET /api/v1/orgs/:slug
pub async fn handle_get_org(
    State(state): State<Arc<AppState>>,
    path: Path<String>,
    query: Query<nucleus_org::handlers::org::ProjectContext>,
) -> Result<Json<serde_json::Value>, AppError> {
    nucleus_org::handlers::org::handle_get_org(State(state.org_service.clone()), path, query).await
}

/// PATCH /api/v1/orgs/:slug
pub async fn handle_update_org(
    State(state): State<Arc<AppState>>,
    path: Path<String>,
    query: Query<nucleus_org::handlers::org::ProjectContext>,
    Json(req): Json<nucleus_org::handlers::org::UpdateOrgRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    nucleus_org::handlers::org::handle_update_org(
        State(state.org_service.clone()),
        path,
        query,
        Json(req),
    )
    .await
}

// ---------------------------------------------------------------------------
// Members
// ---------------------------------------------------------------------------

/// GET /api/v1/orgs/:slug/members
pub async fn handle_list_members(
    State(state): State<Arc<AppState>>,
    path: Path<String>,
    query: Query<nucleus_org::handlers::member::MemberListParams>,
) -> Result<Json<serde_json::Value>, AppError> {
    nucleus_org::handlers::member::handle_list_members(
        State(state.org_service.clone()),
        path,
        query,
    )
    .await
}

/// DELETE /api/v1/orgs/:slug/members/:user_id
pub async fn handle_remove_member(
    State(state): State<Arc<AppState>>,
    path: Path<(String, String)>,
    query: Query<nucleus_org::handlers::member::RemoveMemberParams>,
) -> Result<axum::http::StatusCode, AppError> {
    nucleus_org::handlers::member::handle_remove_member(
        State(state.org_service.clone()),
        path,
        query,
    )
    .await
}

/// PATCH /api/v1/orgs/:slug/members/:user_id/role
pub async fn handle_change_role(
    State(state): State<Arc<AppState>>,
    path: Path<(String, String)>,
    query: Query<nucleus_org::handlers::member::RemoveMemberParams>,
    Json(req): Json<nucleus_org::handlers::member::ChangeRoleRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    nucleus_org::handlers::member::handle_change_role(
        State(state.org_service.clone()),
        path,
        query,
        Json(req),
    )
    .await
}

// ---------------------------------------------------------------------------
// Invitations
// ---------------------------------------------------------------------------

/// GET /api/v1/orgs/:slug/invitations
pub async fn handle_list_invitations(
    State(state): State<Arc<AppState>>,
    path: Path<String>,
    query: Query<nucleus_org::handlers::invitation::ProjectContext>,
) -> Result<Json<serde_json::Value>, AppError> {
    nucleus_org::handlers::invitation::handle_list_invitations(
        State(state.org_service.clone()),
        path,
        query,
    )
    .await
}

/// POST /api/v1/orgs/:slug/invitations
pub async fn handle_create_invitation(
    State(state): State<Arc<AppState>>,
    path: Path<String>,
    query: Query<nucleus_org::handlers::invitation::InvitationContext>,
    Json(req): Json<nucleus_org::handlers::invitation::CreateInvitationRequest>,
) -> Result<Json<nucleus_org::handlers::invitation::InvitationResponse>, AppError> {
    nucleus_org::handlers::invitation::handle_create_invitation(
        State(state.org_service.clone()),
        path,
        query,
        Json(req),
    )
    .await
}

/// POST /api/v1/orgs/:slug/invitations/:id/accept
pub async fn handle_accept_invitation(
    State(state): State<Arc<AppState>>,
    path: Path<(String, String)>,
    query: Query<nucleus_org::handlers::invitation::AcceptInvitationParams>,
) -> Result<Json<serde_json::Value>, AppError> {
    nucleus_org::handlers::invitation::handle_accept_invitation(
        State(state.org_service.clone()),
        path,
        query,
    )
    .await
}

/// DELETE /api/v1/orgs/:slug/invitations/:id
pub async fn handle_revoke_invitation(
    State(state): State<Arc<AppState>>,
    path: Path<(String, String)>,
    query: Query<nucleus_org::handlers::invitation::ProjectContext>,
) -> Result<axum::http::StatusCode, AppError> {
    nucleus_org::handlers::invitation::handle_revoke_invitation(
        State(state.org_service.clone()),
        path,
        query,
    )
    .await
}
