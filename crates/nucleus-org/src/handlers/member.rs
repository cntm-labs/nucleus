use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use nucleus_core::error::AppError;
use nucleus_core::pagination::PaginationParams;
use nucleus_core::types::{ProjectId, RoleId, UserId};
use serde::Deserialize;

use crate::organization::OrgService;

/// Shared query parameters carrying the caller's context (project + user).
#[derive(Debug, Deserialize)]
pub struct RequestContext {
    pub project_id: String,
    pub user_id: String,
}

#[derive(Debug, Deserialize)]
pub struct InviteMemberRequest {
    pub email: String,
    pub role_id: String,
}

#[derive(Debug, Deserialize)]
pub struct ChangeRoleRequest {
    pub role_id: String,
}

/// GET /api/v1/orgs/:slug/members
pub async fn handle_list_members(
    State(org_service): State<Arc<OrgService>>,
    Path(slug): Path<String>,
    Query(ctx): Query<MemberListParams>,
) -> Result<Json<serde_json::Value>, AppError> {
    let project_id: ProjectId = ctx
        .project_id
        .parse()
        .map_err(|e: uuid::Error| AppError::Internal(e.into()))?;

    // Resolve the org by slug to get the org_id
    let org = org_service.get_org_by_slug(&project_id, &slug).await?;

    let params = PaginationParams {
        limit: ctx.limit.unwrap_or(20),
        cursor: ctx.cursor,
    };
    let members = org_service.list_members(&org.id, &params).await?;
    Ok(Json(serde_json::to_value(members).unwrap()))
}

#[derive(Debug, Deserialize)]
pub struct MemberListParams {
    pub project_id: String,
    pub limit: Option<u32>,
    pub cursor: Option<String>,
}

/// POST /api/v1/orgs/:slug/invitations -- invite member
///
/// This is the quick-add path: it directly adds a user as a member (by
/// user_id supplied in the body). For email-based invitations, use the
/// invitation handlers instead.
pub async fn handle_invite_member(
    State(org_service): State<Arc<OrgService>>,
    Path(slug): Path<String>,
    Query(ctx): Query<RequestContext>,
    Json(req): Json<InviteMemberRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let project_id: ProjectId = ctx
        .project_id
        .parse()
        .map_err(|e: uuid::Error| AppError::Internal(e.into()))?;
    let inviter: UserId = ctx
        .user_id
        .parse()
        .map_err(|e: uuid::Error| AppError::Internal(e.into()))?;
    let role_id: RoleId = req
        .role_id
        .parse()
        .map_err(|e: uuid::Error| AppError::Internal(e.into()))?;

    let org = org_service.get_org_by_slug(&project_id, &slug).await?;

    // For now we treat `email` as a user_id (UUID) for direct-add.
    // A full implementation would look up user by email first.
    let member_user_id: UserId = req
        .email
        .parse()
        .map_err(|e: uuid::Error| AppError::Internal(e.into()))?;

    let member = org_service
        .add_member(&org.id, &member_user_id, &role_id, Some(&inviter))
        .await?;
    Ok(Json(serde_json::to_value(member).unwrap()))
}

/// DELETE /api/v1/orgs/:slug/members/:user_id
pub async fn handle_remove_member(
    State(org_service): State<Arc<OrgService>>,
    Path((slug, member_id)): Path<(String, String)>,
    Query(ctx): Query<RemoveMemberParams>,
) -> Result<StatusCode, AppError> {
    let project_id: ProjectId = ctx
        .project_id
        .parse()
        .map_err(|e: uuid::Error| AppError::Internal(e.into()))?;
    let user_id: UserId = member_id
        .parse()
        .map_err(|e: uuid::Error| AppError::Internal(e.into()))?;

    let org = org_service.get_org_by_slug(&project_id, &slug).await?;
    org_service.remove_member(&org.id, &user_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, Deserialize)]
pub struct RemoveMemberParams {
    pub project_id: String,
}

/// PATCH /api/v1/orgs/:slug/members/:user_id/role
pub async fn handle_change_role(
    State(org_service): State<Arc<OrgService>>,
    Path((slug, member_id)): Path<(String, String)>,
    Query(ctx): Query<RemoveMemberParams>,
    Json(req): Json<ChangeRoleRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let project_id: ProjectId = ctx
        .project_id
        .parse()
        .map_err(|e: uuid::Error| AppError::Internal(e.into()))?;
    let user_id: UserId = member_id
        .parse()
        .map_err(|e: uuid::Error| AppError::Internal(e.into()))?;
    let role_id: RoleId = req
        .role_id
        .parse()
        .map_err(|e: uuid::Error| AppError::Internal(e.into()))?;

    let org = org_service.get_org_by_slug(&project_id, &slug).await?;
    org_service
        .update_member_role(&org.id, &user_id, &role_id)
        .await?;

    Ok(Json(serde_json::json!({ "ok": true })))
}
