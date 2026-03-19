use nucleus_core::types::*;
use serde::{Deserialize, Serialize};

/// System-defined roles (created per project)
pub const SYSTEM_ROLE_OWNER: &str = "owner";
pub const SYSTEM_ROLE_ADMIN: &str = "admin";
pub const SYSTEM_ROLE_MEMBER: &str = "member";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    pub id: RoleId,
    pub project_id: ProjectId,
    pub org_id: Option<OrgId>,
    pub name: String,
    pub description: Option<String>,
    pub is_system: bool,
}

#[derive(Debug)]
pub struct NewRole {
    pub project_id: ProjectId,
    pub org_id: Option<OrgId>,
    pub name: String,
    pub description: Option<String>,
    pub is_system: bool,
}
