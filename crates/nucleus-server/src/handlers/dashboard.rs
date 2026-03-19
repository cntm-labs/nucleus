use axum::Json;
use nucleus_core::error::AppError;

// ---------------------------------------------------------------------------
// Phase 5: Dashboard API routes (thin wrappers)
// ---------------------------------------------------------------------------

// -- Projects ---------------------------------------------------------------

/// GET /api/v1/dashboard/projects
pub async fn handle_list_projects() -> Result<Json<serde_json::Value>, AppError> {
    nucleus_admin_api::handlers::dashboard::handle_list_projects().await
}

/// POST /api/v1/dashboard/projects
pub async fn handle_create_project(
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    nucleus_admin_api::handlers::dashboard::handle_create_project(Json(req)).await
}

/// GET /api/v1/dashboard/projects/:id
pub async fn handle_get_project() -> Result<Json<serde_json::Value>, AppError> {
    nucleus_admin_api::handlers::dashboard::handle_get_project().await
}

/// PATCH /api/v1/dashboard/projects/:id
pub async fn handle_update_project(
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    nucleus_admin_api::handlers::dashboard::handle_update_project(Json(req)).await
}

// -- OAuth providers --------------------------------------------------------

/// GET /api/v1/dashboard/projects/:id/providers
pub async fn handle_list_providers() -> Result<Json<serde_json::Value>, AppError> {
    nucleus_admin_api::handlers::dashboard::handle_list_providers().await
}

/// POST /api/v1/dashboard/projects/:id/providers
pub async fn handle_configure_provider(
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    nucleus_admin_api::handlers::dashboard::handle_configure_provider(Json(req)).await
}

/// DELETE /api/v1/dashboard/projects/:id/providers/:provider_id
pub async fn handle_delete_provider() -> Result<axum::http::StatusCode, AppError> {
    nucleus_admin_api::handlers::dashboard::handle_delete_provider().await
}

// -- API keys ---------------------------------------------------------------

/// GET /api/v1/dashboard/projects/:id/api-keys
pub async fn handle_list_api_keys() -> Result<Json<serde_json::Value>, AppError> {
    nucleus_admin_api::handlers::dashboard::handle_list_api_keys().await
}

/// POST /api/v1/dashboard/projects/:id/api-keys
pub async fn handle_create_api_key(
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    nucleus_admin_api::handlers::dashboard::handle_create_api_key(Json(req)).await
}

/// DELETE /api/v1/dashboard/projects/:id/api-keys/:key_id
pub async fn handle_revoke_api_key() -> Result<axum::http::StatusCode, AppError> {
    nucleus_admin_api::handlers::dashboard::handle_revoke_api_key().await
}

// -- Signing keys -----------------------------------------------------------

/// GET /api/v1/dashboard/projects/:id/signing-keys
pub async fn handle_list_signing_keys() -> Result<Json<serde_json::Value>, AppError> {
    nucleus_admin_api::handlers::dashboard::handle_list_signing_keys().await
}

/// POST /api/v1/dashboard/projects/:id/signing-keys/rotate
pub async fn handle_rotate_signing_key() -> Result<Json<serde_json::Value>, AppError> {
    nucleus_admin_api::handlers::dashboard::handle_rotate_signing_key().await
}

// -- Templates --------------------------------------------------------------

/// GET /api/v1/dashboard/projects/:id/templates
pub async fn handle_list_templates() -> Result<Json<serde_json::Value>, AppError> {
    nucleus_admin_api::handlers::dashboard::handle_list_templates().await
}

/// PATCH /api/v1/dashboard/projects/:id/templates/:template_id
pub async fn handle_update_template(
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    nucleus_admin_api::handlers::dashboard::handle_update_template(Json(req)).await
}

/// POST /api/v1/dashboard/projects/:id/templates/:template_id/reset
pub async fn handle_reset_template() -> Result<Json<serde_json::Value>, AppError> {
    nucleus_admin_api::handlers::dashboard::handle_reset_template().await
}

// -- JWT templates ----------------------------------------------------------

/// GET /api/v1/dashboard/projects/:id/jwt-templates
pub async fn handle_list_jwt_templates() -> Result<Json<serde_json::Value>, AppError> {
    nucleus_admin_api::handlers::dashboard::handle_list_jwt_templates().await
}

/// POST /api/v1/dashboard/projects/:id/jwt-templates
pub async fn handle_create_jwt_template(
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    nucleus_admin_api::handlers::dashboard::handle_create_jwt_template(Json(req)).await
}

/// PATCH /api/v1/dashboard/projects/:id/jwt-templates/:jt_id
pub async fn handle_update_jwt_template(
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    nucleus_admin_api::handlers::dashboard::handle_update_jwt_template(Json(req)).await
}

// -- Analytics --------------------------------------------------------------

/// GET /api/v1/dashboard/projects/:id/analytics
pub async fn handle_get_analytics() -> Result<Json<serde_json::Value>, AppError> {
    nucleus_admin_api::handlers::dashboard::handle_get_analytics().await
}

// -- Billing ----------------------------------------------------------------

/// GET /api/v1/dashboard/projects/:id/usage
pub async fn handle_get_usage() -> Result<Json<serde_json::Value>, AppError> {
    nucleus_admin_api::handlers::dashboard::handle_get_usage().await
}

/// GET /api/v1/dashboard/projects/:id/subscription
pub async fn handle_get_subscription() -> Result<Json<serde_json::Value>, AppError> {
    nucleus_admin_api::handlers::dashboard::handle_get_subscription().await
}

// -- Audit logs -------------------------------------------------------------

/// GET /api/v1/dashboard/projects/:id/audit-logs
pub async fn handle_list_audit_logs() -> Result<Json<serde_json::Value>, AppError> {
    nucleus_admin_api::handlers::dashboard::handle_list_audit_logs().await
}

// -- Settings ---------------------------------------------------------------

/// GET /api/v1/dashboard/projects/:id/settings
pub async fn handle_get_settings() -> Result<Json<serde_json::Value>, AppError> {
    nucleus_admin_api::handlers::dashboard::handle_get_settings().await
}

/// PATCH /api/v1/dashboard/projects/:id/settings
pub async fn handle_update_settings(
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    nucleus_admin_api::handlers::dashboard::handle_update_settings(Json(req)).await
}
