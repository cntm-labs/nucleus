use std::sync::Arc;

use crate::core::crypto;
use crate::core::error::{ApiError, AppError};
use crate::core::pagination::PaginationParams;
use crate::core::types::{AccountId, ApiKeyId, ProjectId};
use crate::db::repos::api_key_repo::{ApiKeyRepository, NewApiKey};
use crate::db::repos::audit_repo::AuditRepository;
use crate::db::repos::project_repo::{NewProject, ProjectRepository};
use crate::db::repos::signing_key_repo::SigningKeyRepository;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use chrono::Utc;
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct DashboardState {
    pub project_repo: Arc<dyn ProjectRepository>,
    pub api_key_repo: Arc<dyn ApiKeyRepository>,
    pub audit_repo: Arc<dyn AuditRepository>,
    pub signing_key_repo: Arc<dyn SigningKeyRepository>,
    pub master_key: [u8; 32],
}

// ---------------------------------------------------------------------------
// Projects
// ---------------------------------------------------------------------------

// TODO: extract account_id from authenticated dashboard session
// For now, use Uuid::nil() as placeholder
fn placeholder_account_id() -> AccountId {
    AccountId::from_uuid(Uuid::nil())
}

pub async fn handle_list_projects(
    State(state): State<DashboardState>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<serde_json::Value>, AppError> {
    let account_id = placeholder_account_id();
    let result = state
        .project_repo
        .list_by_account(&account_id, &params)
        .await?;
    Ok(Json(
        serde_json::to_value(&result).map_err(|e| AppError::Internal(e.into()))?,
    ))
}

#[derive(Debug, Deserialize)]
pub struct CreateProjectRequest {
    pub name: String,
    pub slug: String,
    #[serde(default)]
    pub data_mode: Option<String>,
}

pub async fn handle_create_project(
    State(state): State<DashboardState>,
    Json(req): Json<CreateProjectRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let account_id = placeholder_account_id();
    let new_project = NewProject {
        account_id,
        name: req.name,
        slug: req.slug,
        plan_id: Uuid::nil(), // Default free plan
        data_mode: req.data_mode,
    };
    let project = state.project_repo.create(&new_project).await?;
    Ok(Json(
        serde_json::to_value(&project).map_err(|e| AppError::Internal(e.into()))?,
    ))
}

pub async fn handle_get_project(
    State(state): State<DashboardState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let project_id = ProjectId::from_uuid(id);
    let project = state
        .project_repo
        .find_by_id(&project_id)
        .await?
        .ok_or(AppError::Api(ApiError::NotFound))?;
    Ok(Json(
        serde_json::to_value(&project).map_err(|e| AppError::Internal(e.into()))?,
    ))
}

#[derive(Debug, Deserialize)]
pub struct UpdateProjectRequest {
    #[serde(default)]
    pub settings: Option<serde_json::Value>,
}

pub async fn handle_update_project(
    State(state): State<DashboardState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateProjectRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let project_id = ProjectId::from_uuid(id);
    if let Some(settings) = req.settings {
        let project = state
            .project_repo
            .update_settings(&project_id, settings)
            .await?;
        Ok(Json(
            serde_json::to_value(&project).map_err(|e| AppError::Internal(e.into()))?,
        ))
    } else {
        let project = state
            .project_repo
            .find_by_id(&project_id)
            .await?
            .ok_or(AppError::Api(ApiError::NotFound))?;
        Ok(Json(
            serde_json::to_value(&project).map_err(|e| AppError::Internal(e.into()))?,
        ))
    }
}

// ---------------------------------------------------------------------------
// OAuth providers (stub — no provider repo yet)
// ---------------------------------------------------------------------------

pub async fn handle_list_providers() -> Result<Json<serde_json::Value>, AppError> {
    Ok(Json(json!({
        "data": [],
        "has_more": false,
        "total_count": 0
    })))
}

pub async fn handle_configure_provider(
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    Ok(Json(json!({
        "id": Uuid::new_v4().to_string(),
        "provider": req.get("provider").and_then(|v| v.as_str()).unwrap_or("unknown"),
        "enabled": true,
        "created_at": Utc::now().to_rfc3339()
    })))
}

pub async fn handle_delete_provider() -> Result<StatusCode, AppError> {
    Ok(StatusCode::NO_CONTENT)
}

// ---------------------------------------------------------------------------
// API Keys
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct CreateApiKeyRequest {
    pub name: Option<String>,
    #[serde(default)]
    pub scopes: Vec<String>,
    #[serde(default)]
    pub environment: Option<String>,
}

pub async fn handle_list_api_keys(
    State(state): State<DashboardState>,
    Path(project_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let pid = ProjectId::from_uuid(project_id);
    let keys = state.api_key_repo.find_by_project(&pid).await?;
    // Filter out revoked keys from listing
    let active: Vec<_> = keys.iter().filter(|k| k.revoked_at.is_none()).collect();
    Ok(Json(json!({
        "data": active,
        "has_more": false,
        "total_count": active.len()
    })))
}

pub async fn handle_create_api_key(
    State(state): State<DashboardState>,
    Path(project_id): Path<Uuid>,
    Json(req): Json<CreateApiKeyRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let pid = ProjectId::from_uuid(project_id);

    // Generate a real API key
    let raw_key = format!("nk_live_{}", crypto::generate_token());
    let key_hash = crypto::generate_token_hash(&raw_key);
    let key_prefix = format!("{}...", &raw_key[..16]);

    let new_key = NewApiKey {
        project_id: pid,
        key_type: "secret".to_string(),
        key_hash,
        key_prefix: key_prefix.clone(),
        environment: req.environment,
        label: req.name,
        scopes: req.scopes,
        rate_limit: None,
        expires_at: None,
    };

    let api_key = state.api_key_repo.create(&new_key).await?;

    // Return the full key ONCE (it's hashed in DB, can never be retrieved again)
    Ok(Json(json!({
        "id": api_key.id.to_string(),
        "key": raw_key,
        "key_prefix": key_prefix,
        "label": api_key.label,
        "environment": api_key.environment,
        "scopes": api_key.scopes,
        "created_at": api_key.created_at.to_rfc3339()
    })))
}

pub async fn handle_revoke_api_key(
    State(state): State<DashboardState>,
    Path((_project_id, key_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, AppError> {
    let kid = ApiKeyId::from_uuid(key_id);
    state.api_key_repo.revoke(&kid).await?;
    Ok(StatusCode::NO_CONTENT)
}

// ---------------------------------------------------------------------------
// Signing keys
// ---------------------------------------------------------------------------

pub async fn handle_list_signing_keys(
    State(state): State<DashboardState>,
    Path(project_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    // Only find_current is available — return as single-element list
    let current = state.signing_key_repo.find_current(&project_id).await?;
    let data: Vec<serde_json::Value> = match current {
        Some(key) => vec![json!({
            "id": key.id.to_string(),
            "algorithm": key.algorithm,
            "public_key": key.public_key,
            "is_current": key.is_current,
        })],
        None => vec![],
    };
    Ok(Json(json!({
        "data": data,
        "has_more": false,
        "total_count": data.len()
    })))
}

pub async fn handle_rotate_signing_key(
    State(_state): State<DashboardState>,
    Path(_project_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    // Key rotation requires JwtService::generate_key_pair (in nucleus-auth)
    // which cannot be called from nucleus-admin-api due to dependency rules.
    // This will be wired through the server composition layer.
    Err(AppError::Internal(anyhow::anyhow!(
        "Signing key rotation requires server-level key generation (not yet wired)"
    )))
}

// ---------------------------------------------------------------------------
// Templates (stub — no template repo yet)
// ---------------------------------------------------------------------------

pub async fn handle_list_templates() -> Result<Json<serde_json::Value>, AppError> {
    Ok(Json(
        json!({ "data": [], "has_more": false, "total_count": 0 }),
    ))
}

pub async fn handle_update_template(
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    Ok(Json(json!({
        "id": req.get("id").and_then(|v| v.as_str()).unwrap_or("unknown"),
        "name": req.get("name").and_then(|v| v.as_str()).unwrap_or("Updated Template"),
        "body": req.get("body").and_then(|v| v.as_str()).unwrap_or(""),
        "updated_at": Utc::now().to_rfc3339()
    })))
}

pub async fn handle_reset_template() -> Result<Json<serde_json::Value>, AppError> {
    Ok(Json(json!({
        "id": "default",
        "name": "Default Template",
        "body": "",
        "is_default": true,
        "updated_at": Utc::now().to_rfc3339()
    })))
}

// ---------------------------------------------------------------------------
// JWT templates (stub — no JWT template repo yet)
// ---------------------------------------------------------------------------

pub async fn handle_list_jwt_templates() -> Result<Json<serde_json::Value>, AppError> {
    Ok(Json(
        json!({ "data": [], "has_more": false, "total_count": 0 }),
    ))
}

pub async fn handle_create_jwt_template(
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    Ok(Json(json!({
        "id": Uuid::new_v4().to_string(),
        "name": req.get("name").and_then(|v| v.as_str()).unwrap_or("New JWT Template"),
        "claims": req.get("claims").cloned().unwrap_or(json!({})),
        "lifetime": req.get("lifetime").and_then(|v| v.as_u64()).unwrap_or(3600),
        "created_at": Utc::now().to_rfc3339()
    })))
}

pub async fn handle_update_jwt_template(
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    Ok(Json(json!({
        "id": req.get("id").and_then(|v| v.as_str()).unwrap_or("unknown"),
        "name": req.get("name").and_then(|v| v.as_str()).unwrap_or("Updated JWT Template"),
        "claims": req.get("claims").cloned().unwrap_or(json!({})),
        "lifetime": req.get("lifetime").and_then(|v| v.as_u64()).unwrap_or(3600),
        "updated_at": Utc::now().to_rfc3339()
    })))
}

// ---------------------------------------------------------------------------
// Analytics (stub — needs analytics queries)
// ---------------------------------------------------------------------------

pub async fn handle_get_analytics() -> Result<Json<serde_json::Value>, AppError> {
    Ok(Json(json!({
        "total_users": 0,
        "active_users_30d": 0,
        "total_sessions": 0,
        "sign_ups_30d": 0,
        "sign_ins_30d": 0,
        "period_start": Utc::now().to_rfc3339(),
        "period_end": Utc::now().to_rfc3339()
    })))
}

// ---------------------------------------------------------------------------
// Billing (stub — needs billing integration)
// ---------------------------------------------------------------------------

pub async fn handle_get_usage() -> Result<Json<serde_json::Value>, AppError> {
    Ok(Json(json!({
        "plan": "free",
        "mau": 0,
        "mau_limit": 10000,
        "api_calls": 0,
        "api_calls_limit": 100000,
        "period_start": Utc::now().to_rfc3339(),
        "period_end": Utc::now().to_rfc3339()
    })))
}

pub async fn handle_get_subscription() -> Result<Json<serde_json::Value>, AppError> {
    Ok(Json(json!({
        "plan": "free",
        "status": "active",
        "current_period_start": Utc::now().to_rfc3339(),
        "current_period_end": Utc::now().to_rfc3339(),
        "cancel_at_period_end": false
    })))
}

// ---------------------------------------------------------------------------
// Audit Logs
// ---------------------------------------------------------------------------

pub async fn handle_list_audit_logs(
    State(state): State<DashboardState>,
    Path(project_id): Path<Uuid>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<serde_json::Value>, AppError> {
    let pid = ProjectId::from_uuid(project_id);
    let result = state.audit_repo.list_audit_logs(&pid, &params).await?;
    Ok(Json(
        serde_json::to_value(&result).map_err(|e| AppError::Internal(e.into()))?,
    ))
}

// ---------------------------------------------------------------------------
// Settings
// ---------------------------------------------------------------------------

pub async fn handle_get_settings(
    State(state): State<DashboardState>,
    Path(project_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let pid = ProjectId::from_uuid(project_id);
    let project = state
        .project_repo
        .find_by_id(&pid)
        .await?
        .ok_or(AppError::Api(ApiError::NotFound))?;

    Ok(Json(json!({
        "session_ttl": project.session_ttl,
        "jwt_lifetime": project.jwt_lifetime,
        "jwt_algorithm": project.jwt_algorithm,
        "allowed_origins": project.allowed_origins,
        "settings": project.settings,
    })))
}

pub async fn handle_update_settings(
    State(state): State<DashboardState>,
    Path(project_id): Path<Uuid>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    let pid = ProjectId::from_uuid(project_id);

    // Merge submitted settings into existing project settings
    let project = state.project_repo.update_settings(&pid, req).await?;

    Ok(Json(json!({
        "session_ttl": project.session_ttl,
        "jwt_lifetime": project.jwt_lifetime,
        "jwt_algorithm": project.jwt_algorithm,
        "allowed_origins": project.allowed_origins,
        "settings": project.settings,
    })))
}
