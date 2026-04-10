use std::sync::Arc;

use crate::core::error::AppError;
use crate::core::types::{InvitationId, ProjectId, RoleId, UserId};
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::org::invitation::InvitationService;
use crate::org::organization::OrgService;

#[derive(Debug, Deserialize)]
pub struct CreateInvitationRequest {
    pub email: String,
    pub role: Option<String>, // defaults to "member"
}

#[derive(Debug, Serialize)]
pub struct InvitationResponse {
    pub id: String,
    pub email: String,
    pub role: String,
    pub status: String,
    pub expires_at: String,
    pub invite_url: String,
}

/// Query parameters carrying the caller's context.
#[derive(Debug, Deserialize)]
pub struct InvitationContext {
    pub project_id: String,
    pub user_id: String,
    /// Base URL for building invitation links (e.g. "https://nucleus.dev").
    pub base_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ProjectContext {
    pub project_id: String,
}

/// POST /api/v1/orgs/:slug/invitations
pub async fn handle_create_invitation(
    State(org_service): State<Arc<OrgService>>,
    Path(slug): Path<String>,
    Query(ctx): Query<InvitationContext>,
    Json(req): Json<CreateInvitationRequest>,
) -> Result<Json<InvitationResponse>, AppError> {
    let project_id: ProjectId = ctx
        .project_id
        .parse()
        .map_err(|e: uuid::Error| AppError::Internal(e.into()))?;
    let _inviter: UserId = ctx
        .user_id
        .parse()
        .map_err(|e: uuid::Error| AppError::Internal(e.into()))?;

    // Verify the org exists
    let org = org_service.get_org_by_slug(&project_id, &slug).await?;

    // Generate invitation token
    let generated = InvitationService::generate();

    let role = req.role.unwrap_or_else(|| "member".to_string());
    let base_url = ctx
        .base_url
        .unwrap_or_else(|| "https://nucleus.dev".to_string());
    let invite_url = InvitationService::build_url(&base_url, &slug, &generated.invitation_token);

    let response = InvitationResponse {
        id: InvitationId::new().to_string(),
        email: req.email,
        role,
        status: "pending".to_string(),
        expires_at: generated.expires_at.to_rfc3339(),
        invite_url,
    };

    // In a full implementation we would persist the invitation (token_hash,
    // org_id, email, role_id, etc.) to a database table here.
    let _ = org.id; // used for persistence
    let _ = generated.token_hash; // stored alongside the invitation record

    Ok(Json(response))
}

/// POST /api/v1/orgs/:slug/invitations/:id/accept
pub async fn handle_accept_invitation(
    State(org_service): State<Arc<OrgService>>,
    Path((slug, invitation_id)): Path<(String, String)>,
    Query(ctx): Query<AcceptInvitationParams>,
) -> Result<Json<serde_json::Value>, AppError> {
    let project_id: ProjectId = ctx
        .project_id
        .parse()
        .map_err(|e: uuid::Error| AppError::Internal(e.into()))?;
    let user_id: UserId = ctx
        .user_id
        .parse()
        .map_err(|e: uuid::Error| AppError::Internal(e.into()))?;

    let org = org_service.get_org_by_slug(&project_id, &slug).await?;

    // In a full implementation we would:
    // 1. Load the invitation record by `invitation_id`
    // 2. Call InvitationService::verify_token(provided_token, stored_hash, status, expires_at)
    // 3. Add the user as a member with the invitation's role
    // 4. Mark the invitation as accepted
    //
    // For now we parse the role_id from the request and add the member.
    let role_id: RoleId = ctx
        .role_id
        .as_deref()
        .unwrap_or(&RoleId::new().to_string())
        .parse()
        .map_err(|e: uuid::Error| AppError::Internal(e.into()))?;

    let member = org_service
        .add_member(&org.id, &user_id, &role_id, None)
        .await?;

    let _ = invitation_id; // would be used to look up + mark accepted

    Ok(Json(serde_json::to_value(member).unwrap()))
}

#[derive(Debug, Deserialize)]
pub struct AcceptInvitationParams {
    pub project_id: String,
    pub user_id: String,
    pub token: Option<String>,
    pub role_id: Option<String>,
}

/// DELETE /api/v1/orgs/:slug/invitations/:id
pub async fn handle_revoke_invitation(
    State(org_service): State<Arc<OrgService>>,
    Path((slug, _invitation_id)): Path<(String, String)>,
    Query(ctx): Query<ProjectContext>,
) -> Result<StatusCode, AppError> {
    let project_id: ProjectId = ctx
        .project_id
        .parse()
        .map_err(|e: uuid::Error| AppError::Internal(e.into()))?;

    // Verify the org exists
    let _org = org_service.get_org_by_slug(&project_id, &slug).await?;

    // In a full implementation we would load the invitation record by
    // `_invitation_id`, check it belongs to this org, and set its status
    // to InvitationStatus::Revoked.

    Ok(StatusCode::NO_CONTENT)
}

/// GET /api/v1/orgs/:slug/invitations
pub async fn handle_list_invitations(
    State(org_service): State<Arc<OrgService>>,
    Path(slug): Path<String>,
    Query(ctx): Query<ProjectContext>,
) -> Result<Json<serde_json::Value>, AppError> {
    let project_id: ProjectId = ctx
        .project_id
        .parse()
        .map_err(|e: uuid::Error| AppError::Internal(e.into()))?;

    // Verify the org exists
    let _org = org_service.get_org_by_slug(&project_id, &slug).await?;

    // In a full implementation we would query the invitations table filtered
    // by org_id. Return an empty list for now.
    Ok(Json(serde_json::json!({
        "data": [],
        "has_more": false
    })))
}
