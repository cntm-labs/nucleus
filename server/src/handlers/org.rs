use std::sync::Arc;

use crate::core::error::AppError;
use crate::core::pagination::PaginationParams;
use crate::core::types::{ProjectId, RoleId, UserId};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;

use crate::middleware::auth::JwtAuth;
use crate::state::AppState;

fn parse_project_id(claims: &crate::auth::jwt::NucleusClaims) -> Result<ProjectId, AppError> {
    claims
        .aud
        .parse()
        .map_err(|_| AppError::Auth(crate::core::error::AuthError::TokenInvalid))
}

fn parse_user_id(claims: &crate::auth::jwt::NucleusClaims) -> Result<UserId, AppError> {
    claims
        .sub
        .parse()
        .map_err(|_| AppError::Auth(crate::core::error::AuthError::TokenInvalid))
}

// ---------------------------------------------------------------------------
// Phase 4: Organization routes (authenticated via JWT)
// ---------------------------------------------------------------------------

/// GET /api/v1/orgs
pub async fn handle_list_orgs(
    JwtAuth(claims): JwtAuth,
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, AppError> {
    let project_id = parse_project_id(&claims)?;
    let user_id = parse_user_id(&claims)?;

    let orgs = state
        .org_service
        .list_user_orgs(&project_id, &user_id)
        .await?;
    Ok(Json(serde_json::to_value(orgs).unwrap()))
}

/// POST /api/v1/orgs
pub async fn handle_create_org(
    JwtAuth(claims): JwtAuth,
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateOrgRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let project_id = parse_project_id(&claims)?;
    let user_id = parse_user_id(&claims)?;

    let org = state
        .org_service
        .create_org(&project_id, &req.name, &req.slug, &user_id)
        .await?;
    Ok(Json(serde_json::to_value(org).unwrap()))
}

#[derive(Debug, Deserialize)]
pub struct CreateOrgRequest {
    pub name: String,
    pub slug: String,
}

/// GET /api/v1/orgs/:slug
pub async fn handle_get_org(
    JwtAuth(claims): JwtAuth,
    State(state): State<Arc<AppState>>,
    Path(slug): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let project_id = parse_project_id(&claims)?;

    let org = state
        .org_service
        .get_org_by_slug(&project_id, &slug)
        .await?;
    Ok(Json(serde_json::to_value(org).unwrap()))
}

/// PATCH /api/v1/orgs/:slug
pub async fn handle_update_org(
    JwtAuth(claims): JwtAuth,
    State(state): State<Arc<AppState>>,
    Path(slug): Path<String>,
    Json(_req): Json<UpdateOrgRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let project_id = parse_project_id(&claims)?;

    let org = state
        .org_service
        .get_org_by_slug(&project_id, &slug)
        .await?;
    // TODO: apply update fields when OrgService gains an update method
    Ok(Json(serde_json::to_value(org).unwrap()))
}

#[derive(Debug, Deserialize)]
pub struct UpdateOrgRequest {
    pub _name: Option<String>,
}

// ---------------------------------------------------------------------------
// Members
// ---------------------------------------------------------------------------

/// GET /api/v1/orgs/:slug/members
pub async fn handle_list_members(
    JwtAuth(claims): JwtAuth,
    State(state): State<Arc<AppState>>,
    Path(slug): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let project_id = parse_project_id(&claims)?;

    let org = state
        .org_service
        .get_org_by_slug(&project_id, &slug)
        .await?;
    let params = PaginationParams {
        limit: 20,
        cursor: None,
    };
    let members = state.org_service.list_members(&org.id, &params).await?;
    Ok(Json(serde_json::to_value(members).unwrap()))
}

/// DELETE /api/v1/orgs/:slug/members/:user_id
pub async fn handle_remove_member(
    JwtAuth(claims): JwtAuth,
    State(state): State<Arc<AppState>>,
    Path((slug, member_id)): Path<(String, String)>,
) -> Result<StatusCode, AppError> {
    let project_id = parse_project_id(&claims)?;
    let user_id: UserId = member_id
        .parse()
        .map_err(|e: uuid::Error| AppError::Internal(e.into()))?;

    let org = state
        .org_service
        .get_org_by_slug(&project_id, &slug)
        .await?;
    state.org_service.remove_member(&org.id, &user_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// PATCH /api/v1/orgs/:slug/members/:user_id/role
pub async fn handle_change_role(
    JwtAuth(claims): JwtAuth,
    State(state): State<Arc<AppState>>,
    Path((slug, member_id)): Path<(String, String)>,
    Json(req): Json<ChangeRoleRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let project_id = parse_project_id(&claims)?;
    let user_id: UserId = member_id
        .parse()
        .map_err(|e: uuid::Error| AppError::Internal(e.into()))?;
    let role_id: RoleId = req
        .role_id
        .parse()
        .map_err(|e: uuid::Error| AppError::Internal(e.into()))?;

    let org = state
        .org_service
        .get_org_by_slug(&project_id, &slug)
        .await?;
    state
        .org_service
        .update_member_role(&org.id, &user_id, &role_id)
        .await?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

#[derive(Debug, Deserialize)]
pub struct ChangeRoleRequest {
    pub role_id: String,
}

// ---------------------------------------------------------------------------
// Invitations
// ---------------------------------------------------------------------------

/// GET /api/v1/orgs/:slug/invitations
pub async fn handle_list_invitations(
    JwtAuth(claims): JwtAuth,
    State(state): State<Arc<AppState>>,
    Path(slug): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let project_id = parse_project_id(&claims)?;

    let _org = state
        .org_service
        .get_org_by_slug(&project_id, &slug)
        .await?;
    Ok(Json(serde_json::json!({
        "data": [],
        "has_more": false
    })))
}

/// POST /api/v1/orgs/:slug/invitations
pub async fn handle_create_invitation(
    JwtAuth(claims): JwtAuth,
    State(state): State<Arc<AppState>>,
    Path(slug): Path<String>,
    Json(req): Json<CreateInvitationRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let project_id = parse_project_id(&claims)?;
    let _inviter = parse_user_id(&claims)?;

    let org = state
        .org_service
        .get_org_by_slug(&project_id, &slug)
        .await?;

    let generated = crate::org::invitation::InvitationService::generate();
    let role = req.role.unwrap_or_else(|| "member".to_string());
    let base_url = "https://nucleus.dev".to_string();
    let invite_url = crate::org::invitation::InvitationService::build_url(
        &base_url,
        &slug,
        &generated.invitation_token,
    );

    let _ = org.id;
    let _ = generated.token_hash;

    Ok(Json(serde_json::json!({
        "id": crate::core::types::InvitationId::new().to_string(),
        "email": req.email,
        "role": role,
        "status": "pending",
        "expires_at": generated.expires_at.to_rfc3339(),
        "invite_url": invite_url,
    })))
}

#[derive(Debug, Deserialize)]
pub struct CreateInvitationRequest {
    pub email: String,
    pub role: Option<String>,
}

/// POST /api/v1/orgs/:slug/invitations/:id/accept
pub async fn handle_accept_invitation(
    JwtAuth(claims): JwtAuth,
    State(state): State<Arc<AppState>>,
    Path((slug, _invitation_id)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, AppError> {
    let project_id = parse_project_id(&claims)?;
    let user_id = parse_user_id(&claims)?;

    let org = state
        .org_service
        .get_org_by_slug(&project_id, &slug)
        .await?;

    let role_id = RoleId::new();
    let member = state
        .org_service
        .add_member(&org.id, &user_id, &role_id, None)
        .await?;
    Ok(Json(serde_json::to_value(member).unwrap()))
}

/// DELETE /api/v1/orgs/:slug/invitations/:id
pub async fn handle_revoke_invitation(
    JwtAuth(claims): JwtAuth,
    State(state): State<Arc<AppState>>,
    Path((slug, _invitation_id)): Path<(String, String)>,
) -> Result<StatusCode, AppError> {
    let project_id = parse_project_id(&claims)?;

    let _org = state
        .org_service
        .get_org_by_slug(&project_id, &slug)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
