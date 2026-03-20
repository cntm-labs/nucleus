use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::Json;
use nucleus_core::error::AppError;
use nucleus_core::types::{ProjectId, UserId};
use serde::Deserialize;

use crate::organization::OrgService;

#[derive(Debug, Deserialize)]
pub struct CreateOrgRequest {
    pub name: String,
    pub slug: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateOrgRequest {
    pub name: Option<String>,
}

/// GET /api/v1/orgs -- list user's orgs
///
/// In production the project_id and user_id come from JWT claims injected by
/// middleware. We accept them as query parameters here so the handler can be
/// wired up without the auth middleware in place yet.
pub async fn handle_list_orgs(
    State(org_service): State<Arc<OrgService>>,
    Query(params): Query<ListOrgsParams>,
) -> Result<Json<serde_json::Value>, AppError> {
    let project_id: ProjectId = params
        .project_id
        .parse()
        .map_err(|e: uuid::Error| AppError::Internal(e.into()))?;
    let user_id: UserId = params
        .user_id
        .parse()
        .map_err(|e: uuid::Error| AppError::Internal(e.into()))?;

    let orgs = org_service.list_user_orgs(&project_id, &user_id).await?;
    Ok(Json(serde_json::to_value(orgs).unwrap()))
}

#[derive(Debug, Deserialize)]
pub struct ListOrgsParams {
    pub project_id: String,
    pub user_id: String,
}

/// POST /api/v1/orgs -- create org
pub async fn handle_create_org(
    State(org_service): State<Arc<OrgService>>,
    Query(ctx): Query<RequestContext>,
    Json(req): Json<CreateOrgRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let project_id: ProjectId = ctx
        .project_id
        .parse()
        .map_err(|e: uuid::Error| AppError::Internal(e.into()))?;
    let user_id: UserId = ctx
        .user_id
        .parse()
        .map_err(|e: uuid::Error| AppError::Internal(e.into()))?;

    let org = org_service
        .create_org(&project_id, &req.name, &req.slug, &user_id)
        .await?;
    Ok(Json(serde_json::to_value(org).unwrap()))
}

/// GET /api/v1/orgs/:slug -- org details
pub async fn handle_get_org(
    State(org_service): State<Arc<OrgService>>,
    Path(slug): Path<String>,
    Query(ctx): Query<ProjectContext>,
) -> Result<Json<serde_json::Value>, AppError> {
    let project_id: ProjectId = ctx
        .project_id
        .parse()
        .map_err(|e: uuid::Error| AppError::Internal(e.into()))?;

    let org = org_service.get_org_by_slug(&project_id, &slug).await?;
    Ok(Json(serde_json::to_value(org).unwrap()))
}

/// PATCH /api/v1/orgs/:slug -- update org
///
/// OrgService does not currently expose an update method, so we fetch-then-
/// validate here. When an `update_org` method is added to OrgService this
/// handler can delegate to it.
pub async fn handle_update_org(
    State(org_service): State<Arc<OrgService>>,
    Path(slug): Path<String>,
    Query(ctx): Query<ProjectContext>,
    Json(_req): Json<UpdateOrgRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let project_id: ProjectId = ctx
        .project_id
        .parse()
        .map_err(|e: uuid::Error| AppError::Internal(e.into()))?;

    // Verify the org exists (will 404 if not)
    let org = org_service.get_org_by_slug(&project_id, &slug).await?;

    // TODO: apply update fields when OrgService gains an update method
    Ok(Json(serde_json::to_value(org).unwrap()))
}

/// Query parameters shared across endpoints that need project + user context.
#[derive(Debug, Deserialize)]
pub struct RequestContext {
    pub project_id: String,
    pub user_id: String,
}

/// Query parameters for endpoints that only need project context.
#[derive(Debug, Deserialize)]
pub struct ProjectContext {
    pub project_id: String,
}
